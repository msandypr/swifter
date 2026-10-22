<p align="center">
  <img src="static/logo-swifter.png" width="120" height="120" alt="Swifter Logo" />
</p>

<h1 align="center">Swifter</h1>

<p align="center">
  <strong>Portable Windows (<code>.exe</code>) to Native macOS (<code>.app</code>) Packaging Engine</strong><br>
  <em>Designed and created by <a href="https://github.com/msandypr">msandypr</a></em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-macOS%2012.0+-black?style=flat-square&logo=apple" alt="Platform" />
  <img src="https://img.shields.io/badge/Version-v0.1.0-zinc?style=flat-square" alt="Version" />
  <img src="https://img.shields.io/badge/Framework-Tauri%20v2-24c8db?style=flat-square&logo=tauri&logoColor=white" alt="Tauri v2" />
  <img src="https://img.shields.io/badge/Frontend-Svelte%205-ff3e00?style=flat-square&logo=svelte&logoColor=white" alt="Svelte 5" />
  <img src="https://img.shields.io/badge/Backend-Rust%202024-black?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/License-MIT-gray?style=flat-square" alt="License" />
</p>

---

## Overview

**Swifter** is an ultra-lightweight, high-speed macOS utility built with **Tauri v2 (Rust + Svelte 5 + Tailwind CSS)**. It packages standalone, DRM-free Windows executables (`.exe`) and indie game folders into native, ready-to-click macOS application bundles (`.app`) without complex Wine configurations, terminal commands, or third-party store launchers.

---

### Swifter vs. Traditional Wine Managers

| Feature | Traditional Tools (Porting Kit, Crossover) | Swifter |
| :--- | :--- | :--- |
| **Disk Usage** | 1.5 GB – 2.5 GB per game (full virtual C: drive) | **~15 MB – 30 MB** per game (isolated micro-prefix) |
| **Asset Preservation** | Often isolates only `.exe`, breaking Unity `*_Data` | **Wraps full parent game directory automatically** |
| **Icon Extraction** | Generic placeholder icons or manual PNG upload | **Direct PE binary extraction to Apple `.icns`** |
| **Setup Process** | Multi-step bottle creation, Windows installers | **Drag-and-Drop single-click packaging** |
| **Save Data Access** | Hidden deep inside virtual C: drive trees | **One-click "Save Data" button on Shelf** |
| **UI Aesthetics** | Complex legacy menus and colorful bloat | **Minimalist dark monochrome native macOS design** |

---

## Key Capabilities

### 1. Zero-Config Drag & Drop
Drop an `.exe` file or an entire game folder. Swifter inspects the directory structure, identifies the main executable, and prepares the bundle automatically.

### 2. Whole-Folder Context Preservation
Indie games rely on companion assets surrounding the executable. Swifter ensures Unity data folders (`*_Data`), Godot packages (`.pck`), GameMaker archives (`data.win`), and companion DLLs remain intact.

### 3. Native PE Icon Pipeline
Directly parses Windows Portable Executable headers in Rust via `pelite`. It extracts embedded `RT_GROUP_ICON` resources, scales them across standard Apple resolutions, and writes a native multi-layer Apple `.icns` container (16px to 1024px).

### 4. Isolated Micro-Prefix Strategy (~15–30 MB)
Rather than provisioning a redundant 2 GB virtual C: drive for every single game, Swifter creates an isolated micro-prefix:
- Core Windows libraries (`system32`, `syswow64`, `fonts`) are symlinked to a shared base runtime.
- User-specific files (`AppData/Local`, `AppData/Roaming`, `Saved Games`, `user.reg`) remain strictly isolated per game.

### 5. Multi-Backend Translation Matrix
Swifter allows switching between graphics translation layers depending on the game engine:

| Backend | Target APIs | Translation Layer | Recommended Engine |
| :--- | :--- | :--- | :--- |
| **DXVK** | DirectX 9, 10, 11 | Vulkan via MoltenVK | Unity 3D, Godot, and modern 3D indie titles |
| **Apple D3DMetal** | DirectX 11, 12 | Apple Metal (GPTK 2) | Demanding 3D titles on Apple Silicon |
| **Vanilla Wine** | DirectX 9, OpenGL | Built-in `wined3d` | 2D GameMaker titles, RPG Maker, utilities |

### 6. In-App 1-Click Wine Runtime Downloader
Non-technical users don't need to open the terminal. Swifter automatically checks if a Wine runtime exists on the Mac. If missing, users can click **Download Runtime** to download and configure a pre-built portable runtime with real-time download speed, progress percentage, and automatic extraction.

### 7. Game Shelf & Save Data Management
The **Shelf** tab tracks all packaged applications. Users can launch games, reveal them in Finder, or immediately jump into the game's isolated `AppData` folder to backup or modify save files.

---

## System Architecture

```text
swifter/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Tauri macOS application entrypoint
│   │   ├── lib.rs               # Plugin initialization & command dispatcher
│   │   ├── commands.rs          # IPC bridge (inspect, pack, dry-run, runtime)
│   │   ├── analyzer/
│   │   │   ├── pe_parser.rs     # 32/64-bit detection, metadata, anti-cheat detection
│   │   │   ├── folder_scan.rs   # Engine detection (Unity, Godot, GameMaker, Unreal)
│   │   │   └── icon_extract.rs  # PE icon extraction to multi-resolution .icns
│   │   └── engine/
│   │       ├── prefix_mgr.rs    # Isolated micro-prefix provisioning (~15MB/app)
│   │       ├── launcher_shim.rs # POSIX launcher script with multi-runtime discovery
│   │       ├── bundle_builder.rs# macOS .app bundle assembler (Info.plist, PkgInfo)
│   │       └── recipes.rs       # Engine translation presets & environment configurations
├── src/                         # Svelte 5 Frontend
│   ├── routes/
│   │   ├── +layout.svelte       # App shell & titlebar region
│   │   └── +page.svelte         # Primary dashboard with centered unified navbar
│   └── components/
│       ├── Dropzone.svelte      # Drag-and-drop landing area
│       ├── GameInspector.svelte # Configuration, backend switcher, and dry-run tester
│       ├── ProgressModal.svelte # Visual step-by-step packing progress
│       ├── Shelf.svelte         # Game library with launch & save data access
│       ├── Guide.svelte         # Built-in step-by-step user guide
│       └── About.svelte         # Project details & credits to msandypr
└── recipes/
    └── default_recipes.json     # Heuristic presets and environment definitions
```

---

## Installation & Prerequisites

### System Requirements
- **Operating System:** macOS 12.0 (Monterey) or newer.
- **Architecture:** Apple Silicon (M1/M2/M3/M4) or Intel (x86_64).
- **Wine Runtime:** Swifter creates the macOS `.app` bundle and prefix. To execute the game, an underlying Wine runtime is needed. Swifter can:
  - Download and install an internal runtime automatically via the in-app downloader.
  - Automatically detect **Whisky**, **CrossOver**, or **Homebrew Wine**:
    ```bash
    brew install --cask wine-stable
    ```

---

## Developer Guide

### Setup & Dependencies

Ensure you have **Node/Bun** and **Rust** installed:

```bash
# Check toolchain
cargo --version
bun --version
```

Clone the repository and install frontend dependencies:

```bash
git clone https://github.com/msandypr/swifter.git
cd swifter
bun install
```

### Development Mode

Run the app with live hot-reloading (Vite + Tauri):

```bash
bun run tauri dev
```

### Production Build

Compile optimized release binaries (`.app` and `.dmg`):

```bash
bun run tauri build
```

The compiled assets will be located in:
- `src-tauri/target/release/bundle/macos/Swifter.app`
- `src-tauri/target/release/bundle/dmg/Swifter_0.1.0_aarch64.dmg`

### Automated Tests & Quality Checks

Run the Rust test suite (unit tests and end-to-end integration tests):

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Run frontend type checking:

```bash
bun run check
```

---

## User Workflow

1. **Launch Swifter.**
2. **Drop an `.exe` or game folder:** Drag your game into the dropzone.
3. **Inspect & Tweak:**
   - Verify the detected game engine and architecture.
   - Select the preferred graphics backend (**DXVK**, **D3DMetal**, or **Vanilla**).
   - Choose between **Symlink** (saves disk space) or **Standalone Bundle** (copies all assets inside `.app`).
4. **Test Run (Optional):** Click **Dry Run (3s Test)** to diagnose potential startup logs before packing.
5. **Pack:** Click **Pack with Swifter**. The `.app` bundle is assembled in `/Applications` (or `~/Applications`).
6. **Play & Manage:** Launch the game directly from macOS Finder, Spotlight, Dock, or the Swifter **Shelf**.

---

## Author & Credits

- **Developer:** [msandypr](https://github.com/msandypr)
- **Repository:** [https://github.com/msandypr/swifter](https://github.com/msandypr/swifter)

---

## License

Released under the **[MIT License](LICENSE)**. Open source and free for the indie gaming community.
