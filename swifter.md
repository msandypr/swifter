# TECHNICAL SPECIFICATION & EXECUTION BLUEPRINT: SWIFTER (v2)
## Portable Windows (.exe) to Native macOS (.app) Packaging Engine

## 1. System Overview & Objective

**Swifter** adalah utilitas macOS open-source, ultra-ringan, dan berkecepatan tinggi yang dibangun menggunakan **Tauri v2 (Rust + Svelte/Tailwind)**. Swifter dirancang untuk mengubah binary Windows Portable Executable (`.exe`)—khususnya game DRM-free mandiri dari **itch.io**, developer indie, maupun software utilitas—menjadi bundle aplikasi macOS native (`.app`) siap klik tanpa perlu konfigurasi manual, Wine manager rumit, atau launcher toko pihak ketiga seperti Steam.

Swifter mengotomasi:
1. **Drag-and-Drop Zero-Config:** Pengguna cukup melempar file `.exe` atau folder game.
2. **Whole-Folder Context Preservation:** Menjaga keutuhan aset game indie (folder data Unity, library DLL, `.pck` Godot) dengan membungkus folder induk game secara utuh.
3. **Ekstraksi Icon & Metadata Otomatis:** Membaca header PE untuk menarik icon beresolusi tinggi langsung menjadi format `.icns` native macOS.
4. **Isolated Micro-Prefix:** Alih-alih membuat virtual C: drive 2 GB per game, Swifter menggunakan symlink runtime global bersama direktori user terisolasi (~15–30 MB per game).
5. **Dynamic Translation Engine:** Mendukung Apple Game Porting Toolkit 2 (D3DMetal/Metal), DXVK (DirectX 9–11 via MoltenVK), dan vanilla Wine.

---

## 2. Technical Stack & Dependencies

### Frontend Layer
* **Framework:** Tauri v2 with Svelte 5 (or React 19) + Vite
* **Styling:** Tailwind CSS + Lucide Icons
* **Theme & UI:** Native macOS frosted glassmorphism (`backdrop-blur-md`)

### Backend Layer (Rust 2024 Edition)
Tambahkan dependensi berikut pada `src-tauri/Cargo.toml`:

```toml
[package]
name = "swifter"
version = "2.0.0"
edition = "2024"

[dependencies]
tauri = { version = "2.0", features = ["macos-private-api"] }
tauri-plugin-shell = "2.0"
tauri-plugin-fs = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
pelite = "0.10"       # Parsing PE header, metadata produk, dan resource binary Windows
image = "0.25"        # Manipulasi dan scaling bitmap icon
icns = "0.3"          # Packing bitmap ke format Apple .icns
walkdir = "2.4"       # Traversal direktori folder game itch.io
regex = "1.10"
dirs = "5.0"
slugify = "0.1.0"
```

---

## 3. Directory Layout & Architecture

```text
swifter/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── analyzer/
│   │   │   ├── mod.rs
│   │   │   ├── pe_parser.rs     // Deteksi arsitektur (x86/x64), metadata nama, cek anti-cheat
│   │   │   ├── folder_scan.rs   // Deteksi konteks folder indie (Unity/Godot/GameMaker)
│   │   │   └── icon_extract.rs  // Ekstraksi RT_GROUP_ICON -> .icns multi-resolusi
│   │   ├── engine/
│   │   │   ├── mod.rs
│   │   │   ├── prefix_mgr.rs    // Symlink engine micro-prefix (~15MB per game)
│   │   │   ├── bundle_builder.rs// Pembentukan folder .app dan Info.plist
│   │   │   ├── recipes.rs       // Database aturan auto-tweak (D3DMetal / DXVK)
│   │   │   └── launcher_shim.rs // Generate executable launcher POSIX
│   │   └── commands.rs          // Tauri IPC Bridge
├── src/                         // Frontend UI
│   ├── App.svelte
│   ├── components/
│   │   ├── Dropzone.svelte      // Area drag-and-drop .exe / folder
│   │   ├── GameInspector.svelte // Tampilan ringkasan game & pilihan engine
│   │   ├── ProgressModal.svelte // Visualisasi proses packing
│   │   └── Shelf.svelte         // Daftar game terpasang
│   └── lib/
│       └── tauri-bridge.ts
└── recipes/
    └── default_recipes.json     // Heuristic fallback & preset game
```

---

## 4. Core Module Specifications

### Module 4.1: PE Binary & Indie Folder Analyzer (`analyzer/`)
Game itch.io hampir selalu didistribusikan dalam folder zip terurai. File `.exe` membutuhkan file pendukung di sekitarnya.

1. **Path Resolution:** Jika pengguna menjatuhkan file `game.exe`, tentukan `game_root_dir = parent_dir(game.exe)`.
2. **Engine Detection:**
   * Jika ada `UnityPlayer.dll` atau folder `*_Data/` $\rightarrow$ Tandai sebagai **Unity Game** (Rekomendasi: DXVK).
   * Jika ada file ekstensi `.pck` $\rightarrow$ Tandai sebagai **Godot Game** (Rekomendasi: DXVK/Vulkan).
   * Jika ada `data.win` $\rightarrow$ Tandai sebagai **GameMaker Game** (Rekomendasi: DXVK / Vanilla Wine).
3. **PE Header Inspection (`pelite`):**
   * Periksa `is_64bit` (PE32+ vs PE32).
   * Ambil metadata `FileDescription` atau `ProductName` untuk default nama aplikasi.
4. **Anti-Cheat Guard:** Tolak eksekusi jika menemukan driver anti-cheat tingkat kernel (`vgk.sys`, `EasyAntiCheat.sys`, `BEDaisy.sys`).

```rust
#[derive(serde::Serialize, Clone, Debug)]
pub struct SwifterInspection {
    pub target_exe_path: String,
    pub game_folder_path: String,
    pub app_display_name: String,
    pub is_64bit: bool,
    pub detected_engine: String, // "Unity" | "Godot" | "GameMaker" | "Generic"
    pub recommended_backend: String, // "DXVK" | "D3DMetal" | "Vanilla"
    pub is_anticheat_blocked: bool,
}
```

### Module 4.2: Icon Pipeline (`analyzer/icon_extract.rs`)
1. Baca stream resource `RT_GROUP_ICON` dan `RT_ICON` dari binary Windows menggunakan `pelite`.
2. Ambil frame icon resolusi tertinggi (biasanya 256x256 PNG/BMP).
3. Gunakan crate `image` untuk resize ke layer Apple standar: 16, 32, 64, 128, 256, 512, 1024 px.
4. Simpan ke container `.icns` via crate `icns` di temporary path: `~/.cache/swifter/icons/<hash>.icns`.

### Module 4.3: Isolated Micro-Prefix Strategy (`engine/prefix_mgr.rs`)
**Prinsip Utama:** Menghindari pemborosan disk.
* **Shared Base Runtime:** `$HOME/Library/Application Support/Swifter/runtimes/wine-shared`
* **Isolated Game Prefix:** `$HOME/Library/Application Support/Swifter/prefixes/<app-slug>/`
* Folder `system32`, `syswow64`, dan core file Windows di-symlink ke base runtime.
* Folder `drive_c/users/<user>/AppData` dan file konfigurasi registry (`user.reg`) dibuat independen per game sehingga save data itch.io tidak tercampur dengan game lain.

### Module 4.4: Bundle Assembly & Content Linker (`engine/bundle_builder.rs`)
Membentuk struktur macOS Application Bundle di `/Applications/<AppName>.app`:

```text
/Applications/<AppName>.app/
├── Contents/
│   ├── Info.plist
│   ├── PkgInfo
│   ├── MacOS/
│   │   └── launcher            <- Native Mach-O wrapper / executable POSIX script
│   └── Resources/
│       ├── AppIcon.icns
│       ├── swifter.config.json <- Konfigurasi backend, env vars, dan switch Retina
│       └── game_payload/       <- Symlink atau salinan direktori game asli
```

**Template `Info.plist`:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>launcher</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.swifter.app.{{app_slug}}</string>
    <key>CFBundleName</key>
    <string>{{app_name}}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
```

**Spesifikasi Script `launcher`:**
1. Mengidentifikasi lokasi direktori bundle (`DIR="$(cd "$(dirname "$0")/.." && pwd)"`).
2. Membaca flag dari `Resources/swifter.config.json`.
3. Menetapkan environment variables sesuai backend:
   * Jika `D3DMetal`: export `WINEESYNC=1`, `WINEMSYNC=1`.
   * Jika `DXVK`: inject DLL override `d3d11=n,b`, `dxgi=n,b`.
4. Mengarahkan working directory (`cd`) ke folder game payload agar asset loader game indie dapat menemukan file datanya secara relatif.
5. Mengeksekusi binary `wine64 "$EXE_PATH" "$@"`.

---

## 5. Tauri IPC Command Endpoints (`commands.rs`)

Frontend berkomunikasi dengan Rust backend melalui antarmuka berikut:

```rust
// 1. Menganalisis file .exe atau folder game yang di-drop
#[tauri::command]
async fn inspect_target(dropped_path: String) -> Result<SwifterInspection, String>;

// 2. Mengeksekusi perakitan bundle .app
#[tauri::command]
async fn pack_to_app(
    game_folder_path: String,
    target_exe_name: String,
    app_name: String,
    backend_choice: String, // "DXVK" | "D3DMetal" | "Vanilla"
    enable_msync: bool,
    copy_files_instead_of_symlink: bool
) -> Result<String, String>;

// 3. Dry-run test selama 3 detik untuk menangkap crash log Wine
#[tauri::command]
async fn dry_run_check(exe_path: String, backend: String) -> Result<Vec<String>, String>;

// 4. Membuka folder prefix game untuk mengakses file save data
#[tauri::command]
async fn reveal_save_folder(app_slug: String) -> Result<(), String>;
```

---

## 6. Execution Steps for the Coding Agent

Urutan eksekusi untuk agent koding:

1. **Inisialisasi Project:** Jalankan `bun create tauri-app@latest swifter` dengan template Svelte + Tailwind.
2. **Setup Cargo:** Salin konfigurasi dependensi di Section 2 ke `src-tauri/Cargo.toml`.
3. **Implementasi Modul `analyzer`:**
   * Bangun `pe_parser.rs` untuk inspeksi 32/64 bit dan metadata nama produk via `pelite`.
   * Bangun `folder_scan.rs` untuk deteksi file pendukung game indie (Unity/Godot/data folder).
   * Bangun `icon_extract.rs` untuk parsing icon PE menjadi container `.icns`.
4. **Implementasi Modul `engine`:**
   * Tulis `prefix_mgr.rs` untuk alokasi micro-prefix terisolasi.
   * Tulis `bundle_builder.rs` untuk men-generate folder `/Applications/<AppName>.app`, `Info.plist`, dan file launcher berizin eksekusi (`chmod +x`).
5. **Implementasi UI:** Hubungkan event drop file pada `Dropzone.svelte` ke IPC command `inspect_target` dan tampilkan tombol satu-klik **"Pack with Swifter"**.