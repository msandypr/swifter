# Swifter

> **Portable Windows (`.exe`) to Native macOS (`.app`) Packaging Engine**  
> *Created by [msandypr](https://github.com/msandypr)*

**Swifter** is a lightweight, open-source macOS utility built with **Tauri v2 (Rust + Svelte 5 + Tailwind CSS)**. It converts DRM-free standalone Windows executables (`.exe`) and indie game packages into native, ready-to-click macOS application bundles (`.app`) without manual Wine configuration or launcher bloat.

---

## Key Features

- **Drag-and-Drop Zero-Config:** Drop a Windows `.exe` or an entire unpacked game directory.
- **Whole-Folder Context Preservation:** Keeps companion files intact (Unity `*_Data` directories, Godot `.pck` archives, GameMaker `data.win`, and Windows DLLs).
- **Native PE Icon Extraction:** Automatically parses Windows PE headers directly in Rust and converts embedded `RT_GROUP_ICON` resources into full-resolution Apple `.icns` files (16px to 1024px).
- **Isolated Micro-Prefixes (~15–30 MB):** Avoids duplicating 2 GB virtual C: drives for every game. Reusable Windows system files are symlinked to a shared base runtime, while `AppData`, saved games, and registry keys remain completely isolated per game.
- **Dynamic Translation Backends:**
  - **DXVK:** DirectX 9, 10, and 11 translation via MoltenVK (Vulkan). Recommended for 3D indie titles.
  - **Apple D3DMetal:** DirectX 11 and 12 translation via Apple Game Porting Toolkit 2.
  - **Vanilla Wine:** Direct translation for 2D software and lightweight utilities.
- **One-Click In-App Runtime Downloader:** Automatically checks whether a Wine runtime is installed on the user's Mac and provides a one-click automated download & setup of a pre-built portable Wine runtime into Swifter's shared directory.
- **Save Data Management:** One-click access to game save files in `AppData` directly from the **Shelf** tab.
- **Clean Monochrome Interface:** Minimalist dark neutral gray macOS design with zero distracting color gradients.

---

## Architecture Overview

```text
swifter/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Application entrypoint
│   │   ├── lib.rs               # Tauri plugin initialization & command registration
│   │   ├── commands.rs          # IPC bridge for UI communication
│   │   ├── analyzer/
│   │   │   ├── pe_parser.rs     # 32/64-bit detection, metadata, anti-cheat detection
│   │   │   ├── folder_scan.rs   # Heuristic game engine detection (Unity, Godot, GameMaker)
│   │   │   └── icon_extract.rs  # RT_GROUP_ICON extraction to multi-resolution .icns
│   │   └── engine/
│   │       ├── prefix_mgr.rs    # Isolated micro-prefix manager (~15MB per game)
│   │       ├── launcher_shim.rs # POSIX launcher script with automatic Wine discovery
│   │       ├── bundle_builder.rs# macOS .app bundle assembler & permission manager
│   │       └── recipes.rs       # Engine translation presets & environment configurations
├── src/                         # Svelte 5 Frontend
│   ├── routes/+page.svelte      # Main dashboard (Packer, Shelf, Guide, About)
│   └── components/
│       ├── Dropzone.svelte      # Drag-and-drop zone for .exe or game folder
│       ├── GameInspector.svelte # Game metadata summary, engine selector, and dry-run test
│       ├── ProgressModal.svelte # Visual step-by-step packaging progress
│       ├── Shelf.svelte         # Library of packaged games with launch & save data actions
│       ├── Guide.svelte         # Detailed user guide & instructions
│       └── About.svelte         # Project info & credits to msandypr
└── recipes/
    └── default_recipes.json     # Heuristic presets and backend environment definitions
```

---

## Prerequisites

1. **macOS 12.0 or newer** (Apple Silicon or Intel).
2. **Wine Runtime:** Swifter creates the native macOS wrapper and isolated prefixes. To run the Windows binaries, an underlying Wine runtime is required:
   ```bash
   # Recommended via Homebrew:
   brew install --cask wine-stable
   ```
   *Swifter automatically detects Wine installed via Homebrew, Whisky, CrossOver, or system PATH.*
3. **Build Tools (for developers):**
   - [Rust](https://rustup.rs/) (1.80+)
   - [Bun](https://bun.sh/) (1.0+)

---

## Getting Started

### Development Mode

To start the Vite frontend with hot-reload and compile the Tauri backend:

```bash
bun install
bun run tauri dev
```

### Production Build

To compile an optimized, native macOS release bundle (`.app` and `.dmg`):

```bash
bun run tauri build
```

The resulting artifacts will be available in:
- `src-tauri/target/release/bundle/macos/Swifter.app`
- `src-tauri/target/release/bundle/dmg/Swifter_0.1.0_aarch64.dmg`

### Running Tests

To run the unit and end-to-end integration test suite:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

To type-check the Svelte 5 frontend:

```bash
bun run check
```

---

## Usage Guide

1. **Launch Swifter.**
2. **Drop your game:** Drag and drop a `.exe` file or unpacked game folder into the dropzone.
3. **Review settings:** Swifter automatically detects the game engine, recommends the best translation backend (DXVK, D3DMetal, or Vanilla), and extracts the application icon.
4. **Dry Run (Optional):** Click **Dry Run (3s Test)** to diagnose early launch logs.
5. **Pack:** Click **Pack with Swifter**. Your new `.app` bundle is generated in `/Applications` (or `~/Applications`) ready to launch.
6. **Access Save Files:** Open the **Shelf** tab and click **Save Data** to jump directly to the game's isolated `AppData` folder.

---

## Author & License

Created by **[msandypr](https://github.com/msandypr)**.  
Released under the [MIT License](LICENSE).
