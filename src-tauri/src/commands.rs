use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

use crate::analyzer::{inspect_target_path, SwifterInspection};
use crate::engine::bundle_builder::{self, BuildAppOptions};
use crate::engine::prefix_mgr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledAppInfo {
    pub name: String,
    pub slug: String,
    pub bundle_path: String,
    pub backend: String,
    pub icon_path: Option<String>,
}

/// 1. Inspects a dropped .exe file or game folder
#[tauri::command]
pub async fn inspect_target(dropped_path: String) -> Result<SwifterInspection, String> {
    inspect_target_path(&dropped_path)
}

/// 2. Assembles the native macOS .app bundle
#[tauri::command]
pub async fn pack_to_app(
    game_folder_path: String,
    target_exe_name: String,
    app_name: String,
    backend_choice: String,
    enable_msync: bool,
    copy_files_instead_of_symlink: bool,
) -> Result<String, String> {
    let folder = PathBuf::from(&game_folder_path);

    // Calculate relative path of target exe from game_folder_path
    let target_exe = if Path::new(&target_exe_name).is_absolute() {
        let p = PathBuf::from(&target_exe_name);
        p.strip_prefix(&folder)
            .map(|rel| rel.to_string_lossy().to_string())
            .unwrap_or(target_exe_name)
    } else {
        target_exe_name
    };

    let options = BuildAppOptions {
        game_folder_path: folder,
        target_exe_relative: target_exe,
        app_name,
        backend_choice,
        enable_msync,
        enable_esync: true,
        enable_retina: true,
        copy_files_instead_of_symlink,
        output_dir: None,
        cached_icns_path: None,
    };

    let bundle_path = bundle_builder::build_application_bundle(options)?;
    Ok(bundle_path.to_string_lossy().to_string())
}

/// 3. Performs a 3-second dry-run execution using Wine to capture early crash logs
#[tauri::command]
pub async fn dry_run_check(exe_path: String, backend: String) -> Result<Vec<String>, String> {
    let path = PathBuf::from(&exe_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", exe_path));
    }

    let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));

    // Find wine binary
    let wine_candidates = [
        "/opt/homebrew/bin/wine64",
        "/usr/local/bin/wine64",
        "/opt/homebrew/bin/wine",
        "/usr/local/bin/wine",
    ];

    let mut wine_bin = "wine64".to_string();
    for candidate in wine_candidates {
        if Path::new(candidate).exists() {
            wine_bin = candidate.to_string();
            break;
        }
    }

    let dry_run_prefix = std::env::temp_dir().join("swifter_dry_run_prefix");
    let _ = std::fs::create_dir_all(&dry_run_prefix);

    let mut cmd = Command::new(&wine_bin);
    cmd.arg(&path)
        .current_dir(parent_dir)
        .env("WINEPREFIX", &dry_run_prefix)
        .env("WINEDEBUG", "+err");

    if backend == "D3DMetal" {
        cmd.env("WINEMSYNC", "1");
        cmd.env("WINEESYNC", "1");
    } else if backend == "DXVK" {
        cmd.env("WINEDLLOVERRIDES", "d3d11=n,b;dxgi=n,b");
    }

    // Execute with a 3-second timeout via tokio
    let run_task = tokio::task::spawn_blocking(move || {
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return vec![format!("Failed to spawn Wine: {}. Please verify Wine is installed.", e)],
        };

        // Sleep 3 seconds while letting Wine initialize
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Kill process after 3-second dry-run
        let _ = child.kill();
        let output = child.wait_with_output();

        let mut lines = Vec::new();
        lines.push("=== Dry-run completed (3 seconds test) ===".to_string());
        if let Ok(out) = output {
            let stderr_str = String::from_utf8_lossy(&out.stderr);
            for line in stderr_str.lines().take(50) {
                if !line.trim().is_empty() {
                    lines.push(line.to_string());
                }
            }
        }
        lines
    });

    match tokio::time::timeout(std::time::Duration::from_secs(5), run_task).await {
        Ok(Ok(logs)) => Ok(logs),
        _ => Ok(vec!["Dry run completed or timed out cleanly.".to_string()]),
    }
}

/// 4. Opens the user AppData / Save Game folder in macOS Finder
#[tauri::command]
pub async fn reveal_save_folder(app_slug: String) -> Result<(), String> {
    let save_dir = prefix_mgr::get_save_data_dir(&app_slug);
    let _ = std::fs::create_dir_all(&save_dir);

    Command::new("open")
        .arg(&save_dir)
        .spawn()
        .map_err(|e| format!("Failed to open folder in Finder: {}", e))?;

    Ok(())
}

/// 5. Lists all Swifter-packaged apps in /Applications and ~/Applications
#[tauri::command]
pub async fn list_installed_apps() -> Result<Vec<InstalledAppInfo>, String> {
    let mut apps = Vec::new();

    let search_roots = [
        PathBuf::from("/Applications"),
        dirs::home_dir().map(|h| h.join("Applications")).unwrap_or_default(),
    ];

    for root in search_roots {
        if !root.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("app") {
                    let config_path = path.join("Contents/Resources/swifter.config.json");
                    if config_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&config_path) {
                            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&content) {
                                let name = cfg["app_name"].as_str().unwrap_or("App").to_string();
                                let slug = cfg["app_slug"].as_str().unwrap_or("app").to_string();
                                let backend = cfg["backend"].as_str().unwrap_or("DXVK").to_string();
                                let icon = path.join("Contents/Resources/AppIcon.icns");
                                let icon_path = if icon.exists() {
                                    Some(icon.to_string_lossy().to_string())
                                } else {
                                    None
                                };

                                apps.push(InstalledAppInfo {
                                    name,
                                    slug,
                                    bundle_path: path.to_string_lossy().to_string(),
                                    backend,
                                    icon_path,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(apps)
}

/// 6. Launches an installed macOS application bundle
#[tauri::command]
pub async fn launch_installed_app(bundle_path: String) -> Result<(), String> {
    Command::new("open")
        .arg(&bundle_path)
        .spawn()
        .map_err(|e| format!("Failed to launch {}: {}", bundle_path, e))?;

    Ok(())
}

/// 7. Reveals an application bundle in Finder
#[tauri::command]
pub async fn open_in_finder(path: String) -> Result<(), String> {
    Command::new("open")
        .arg("-R")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to reveal {}: {}", path, e))?;

    Ok(())
}
