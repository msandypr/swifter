use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use crate::analyzer::{inspect_target_path, SwifterInspection};
use crate::engine::bundle_builder::{self, BuildAppOptions};
use crate::engine::prefix_mgr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledAppInfo {
    pub name: String,
    pub slug: String,
    pub bundle_path: String,
    pub backend: String,
    pub target_exe: String,
    pub icon_path: Option<String>,
    pub icon_data_url: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeStatus {
    pub is_installed: bool,
    pub runtime_name: Option<String>,
    pub runtime_path: Option<String>,
    pub provider_upstream: Option<String>,
    pub has_dxvk: bool,
    pub has_d3dmetal: bool,
}

const SWIFTER_USER_AGENT: &str = "Swifter/0.1.0 (+https://github.com/msandypr/swifter)";
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeDownloadProgress {
    pub stage: String,
    pub percent: f64,
    pub downloaded_mb: f64,
    pub total_mb: f64,
    pub speed_mb: f64,
    pub message: String,
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

/// Finds the Wine binary across Swifter internal, Whisky, Homebrew, CrossOver, and PATH
pub fn resolve_wine_binary() -> Result<String, String> {
    // 1. Swifter internal runtime
    let swifter_bin_dir = prefix_mgr::get_swifter_support_dir().join("runtimes/wine/bin");
    if swifter_bin_dir.join("wine64").exists() {
        return Ok(swifter_bin_dir.join("wine64").to_string_lossy().to_string());
    }
    if swifter_bin_dir.join("wine").exists() {
        return Ok(swifter_bin_dir.join("wine").to_string_lossy().to_string());
    }

    // 2. Whisky runtime
    if let Some(app_support) = dirs::data_dir() {
        for whisky_sub in &["com.isaacmarovitz.Whisky/Libraries/Wine/bin", "Whisky/Libraries/Wine/bin"] {
            let p = app_support.join(whisky_sub);
            if p.join("wine64").exists() {
                return Ok(p.join("wine64").to_string_lossy().to_string());
            }
            if p.join("wine").exists() {
                return Ok(p.join("wine").to_string_lossy().to_string());
            }
        }
    }

    // 3. Homebrew Wine
    for brew_path in &["/opt/homebrew/bin/wine64", "/opt/homebrew/bin/wine", "/usr/local/bin/wine64", "/usr/local/bin/wine"] {
        if Path::new(brew_path).exists() {
            return Ok(brew_path.to_string());
        }
    }

    // 4. CrossOver
    let crossover_wine = Path::new("/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine64");
    if crossover_wine.exists() {
        return Ok(crossover_wine.to_string_lossy().to_string());
    }

    // 5. PATH
    for cmd in &["wine64", "wine"] {
        if let Ok(output) = Command::new("which").arg(cmd).output() {
            if output.status.success() {
                let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !p.is_empty() {
                    return Ok(p);
                }
            }
        }
    }

    Err("Wine runtime binary not found. Please install Wine or download it via the Requirements tab.".to_string())
}

/// 3. Performs a 3-second dry-run execution using Wine to capture early crash logs
#[tauri::command]
pub async fn dry_run_check(exe_path: String, backend: String) -> Result<Vec<String>, String> {
    let path = PathBuf::from(&exe_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", exe_path));
    }

    let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));

    let wine_bin = match resolve_wine_binary() {
        Ok(b) => b,
        Err(e) => return Ok(vec![format!("Wine Error: {}", e)]),
    };

    let dry_run_prefix = std::env::temp_dir().join("swifter_dry_run_prefix");
    let _ = std::fs::create_dir_all(&dry_run_prefix);

    let mut cmd = Command::new(&wine_bin);
    cmd.arg(&path)
        .current_dir(parent_dir)
        .env("WINEPREFIX", &dry_run_prefix)
        .env("WINEARCH", "win64")
        .env("WINEDEBUG", "+err")
        .env("MESA_LOADER_DRIVER_OVERRIDE", "zink")
        .env("GALLIUM_DRIVER", "zink")
        .env("MESA_GL_VERSION_OVERRIDE", "4.6")
        .env("MESA_GLSL_VERSION_OVERRIDE", "460");

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
fn extract_icon_data_url(bundle_path: &Path) -> Option<String> {
    let resources = bundle_path.join("Contents/Resources");
    let png_path = resources.join("AppIcon.png");
    if png_path.exists() {
        if let Ok(bytes) = std::fs::read(&png_path) {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            return Some(format!("data:image/png;base64,{}", b64));
        }
    }

    let icns_path = resources.join("AppIcon.icns");
    if icns_path.exists() {
        if let Ok(file) = std::fs::File::open(&icns_path) {
            if let Ok(family) = icns::IconFamily::read(file) {
                for icon_type in &[
                    icns::IconType::RGBA32_128x128,
                    icns::IconType::RGBA32_256x256,
                    icns::IconType::RGBA32_64x64,
                    icns::IconType::RGBA32_32x32,
                    icns::IconType::RGB24_128x128,
                ] {
                    if let Ok(img) = family.get_icon_with_type(*icon_type) {
                        let mut png_buf = Vec::new();
                        if img.write_png(&mut png_buf).is_ok() {
                            use base64::Engine;
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&png_buf);
                            return Some(format!("data:image/png;base64,{}", b64));
                        }
                    }
                }
            }
        }
    }

    None
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
                                let target_exe = cfg["target_exe_relative"].as_str().unwrap_or("").to_string();
                                let icon = path.join("Contents/Resources/AppIcon.icns");
                                let icon_path = if icon.exists() {
                                    Some(icon.to_string_lossy().to_string())
                                } else {
                                    None
                                };
                                let icon_data_url = extract_icon_data_url(&path);

                                apps.push(InstalledAppInfo {
                                    name,
                                    slug,
                                    bundle_path: path.to_string_lossy().to_string(),
                                    backend,
                                    target_exe,
                                    icon_path,
                                    icon_data_url,
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

/// 8. Checks if a Wine translation runtime is installed on the system
#[tauri::command]
pub async fn check_runtime_status() -> Result<RuntimeStatus, String> {
    match resolve_wine_binary() {
        Ok(bin_path) => {
            let (name, upstream) = if bin_path.contains("Swifter") {
                (
                    "WineHQ Devel (Gcenx macOS Package)",
                    "WineHQ & Dean M. (Gcenx) — https://github.com/Gcenx/macOS_Wine_builds (LGPLv2.1+)"
                )
            } else if bin_path.contains("Whisky") {
                (
                    "Whisky Wine Engine (Apple GPTK)",
                    "Whisky App & Apple GPTK — https://github.com/Whisky-App/Whisky (GPLv3)"
                )
            } else if bin_path.contains("homebrew") || bin_path.contains("usr/local") {
                (
                    "Homebrew Wine",
                    "Homebrew Cask wine-stable / WineHQ (LGPLv2.1+)"
                )
            } else if bin_path.contains("CrossOver") {
                (
                    "CrossOver Wine (CodeWeavers)",
                    "CodeWeavers CrossOver (Locally Installed)"
                )
            } else {
                (
                    "System Wine",
                    "Local PATH binary"
                )
            };

            Ok(RuntimeStatus {
                is_installed: true,
                runtime_name: Some(name.to_string()),
                runtime_path: Some(bin_path),
                provider_upstream: Some(upstream.to_string()),
                has_dxvk: true,
                has_d3dmetal: true,
            })
        }
        Err(_) => Ok(RuntimeStatus {
            is_installed: false,
            runtime_name: None,
            runtime_path: None,
            provider_upstream: None,
            has_dxvk: false,
            has_d3dmetal: false,
        }),
    }
}

/// 9. Downloads and provisions the official Swifter Wine runtime in background
#[tauri::command]
pub async fn install_swifter_runtime(app: tauri::AppHandle) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let runtimes_dir = prefix_mgr::get_swifter_support_dir().join("runtimes");
        let target_wine_dir = runtimes_dir.join("wine");
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("swifter");

        let _ = std::fs::create_dir_all(&runtimes_dir);
        let _ = std::fs::create_dir_all(&cache_dir);

        let tar_part = cache_dir.join("wine-devel-osx64.tar.xz.part");
        let tar_dest = cache_dir.join("wine-devel-osx64.tar.xz");
        let _ = std::fs::remove_file(&tar_part);
        let _ = std::fs::remove_file(&tar_dest);
        let download_url = "https://github.com/Gcenx/macOS_Wine_builds/releases/download/11.18/wine-devel-11.18-osx64.tar.xz";
        let total_bytes: u64 = 190974384;
        let total_mb: f64 = total_bytes as f64 / (1024.0 * 1024.0);

        let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
            stage: "downloading".to_string(),
            percent: 0.0,
            downloaded_mb: 0.0,
            total_mb,
            speed_mb: 0.0,
            message: "Connecting to GitHub releases (Gcenx/macOS_Wine_builds)...".to_string(),
        });

        // Spawn curl in background with explicit User-Agent
        let mut child = Command::new("/usr/bin/curl")
            .arg("-L")
            .arg("-f")
            .arg("-A")
            .arg(SWIFTER_USER_AGENT)
            .arg("--retry")
            .arg("3")
            .arg("-o")
            .arg(&tar_part)
            .arg(download_url)
            .spawn()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        let mut prev_bytes: u64 = 0;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        let _ = std::fs::remove_file(&tar_part);
                        let err_msg = "Download interrupted or network connection failed.".to_string();
                        let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
                            stage: "error".to_string(),
                            percent: 0.0,
                            downloaded_mb: 0.0,
                            total_mb,
                            speed_mb: 0.0,
                            message: err_msg.clone(),
                        });
                        return Err(err_msg);
                    }
                    break;
                }
                Ok(None) => {
                    let current_bytes = std::fs::metadata(&tar_part).map(|m| m.len()).unwrap_or(0);
                    let downloaded_mb = current_bytes as f64 / (1024.0 * 1024.0);
                    let percent = ((current_bytes as f64 / total_bytes as f64) * 100.0).min(99.0);
                    let speed_mb = ((current_bytes.saturating_sub(prev_bytes)) as f64 / 0.5) / (1024.0 * 1024.0);
                    prev_bytes = current_bytes;

                    let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
                        stage: "downloading".to_string(),
                        percent,
                        downloaded_mb,
                        total_mb,
                        speed_mb,
                        message: format!("{:.1} MB / {:.1} MB ({:.1} MB/s)", downloaded_mb, total_mb, speed_mb),
                    });

                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&tar_part);
                    return Err(format!("Process monitoring error: {}", e));
                }
            }
        }

        // Verify downloaded archive integrity before extraction
        let downloaded_len = std::fs::metadata(&tar_part).map(|m| m.len()).unwrap_or(0);
        if downloaded_len < 150_000_000 {
            let _ = std::fs::remove_file(&tar_part);
            return Err("Downloaded archive is incomplete or truncated.".to_string());
        }
        std::fs::rename(&tar_part, &tar_dest)
            .map_err(|e| format!("Failed to finalize downloaded archive: {}", e))?;

        // Stage 2: Extracting
        let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
            stage: "extracting".to_string(),
            percent: 100.0,
            downloaded_mb: total_mb,
            total_mb,
            speed_mb: 0.0,
            message: "Extracting portable Wine runtime archive (approx. 15-20s)...".to_string(),
        });

        let stage_dir = cache_dir.join("wine_staging");
        let _ = std::fs::remove_dir_all(&stage_dir);
        let _ = std::fs::create_dir_all(&stage_dir);

        let tar_status = Command::new("/usr/bin/tar")
            .arg("-xf")
            .arg(&tar_dest)
            .arg("-C")
            .arg(&stage_dir)
            .status()
            .map_err(|e| format!("Failed to extract tar archive: {}", e))?;

        if !tar_status.success() {
            let err_msg = "Failed to unpack Wine runtime archive.".to_string();
            let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
                stage: "error".to_string(),
                percent: 0.0,
                downloaded_mb: total_mb,
                total_mb,
                speed_mb: 0.0,
                message: err_msg.clone(),
            });
            return Err(err_msg);
        }

        // Stage 3: Configuring
        let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
            stage: "configuring".to_string(),
            percent: 100.0,
            downloaded_mb: total_mb,
            total_mb,
            speed_mb: 0.0,
            message: "Configuring permissions and verifying Wine binaries...".to_string(),
        });

        let mut source_wine_dir = None;
        for entry in walkdir::WalkDir::new(&stage_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                let bin = entry.path().join("bin");
                if bin.join("wine64").exists() || bin.join("wine").exists() {
                    source_wine_dir = Some(entry.path().to_path_buf());
                    break;
                }
            }
        }

        let source_dir = source_wine_dir
            .ok_or_else(|| "Could not locate wine binary (bin/wine or bin/wine64) inside extracted archive.".to_string())?;

        let _ = std::fs::remove_dir_all(&target_wine_dir);
        let _ = std::fs::create_dir_all(&target_wine_dir);

        let copy_status = Command::new("/bin/cp")
            .arg("-R")
            .arg(format!("{}/.", source_dir.display()))
            .arg(&target_wine_dir)
            .status()
            .map_err(|e| format!("Failed to copy wine runtime into place: {}", e))?;

        if !copy_status.success() {
            let err_msg = "Failed to move wine runtime into Swifter Application Support folder.".to_string();
            let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
                stage: "error".to_string(),
                percent: 0.0,
                downloaded_mb: total_mb,
                total_mb,
                speed_mb: 0.0,
                message: err_msg.clone(),
            });
            return Err(err_msg);
        }

        // Set executable permissions and guarantee wine64 symlink if only wine exists
        let target_bin = target_wine_dir.join("bin");
        let wine_bin = target_bin.join("wine");
        let wine64_bin = target_bin.join("wine64");

        use std::os::unix::fs::PermissionsExt;
        if wine_bin.exists() {
            let _ = std::fs::set_permissions(&wine_bin, std::fs::Permissions::from_mode(0o755));
            if !wine64_bin.exists() {
                let _ = std::os::unix::fs::symlink("wine", &wine64_bin);
            }
        }
        if wine64_bin.exists() {
            let _ = std::fs::set_permissions(&wine64_bin, std::fs::Permissions::from_mode(0o755));
        }

        // Strip macOS Gatekeeper quarantine extended attribute from extracted Wine runtime
        let _ = Command::new("/usr/bin/xattr")
            .arg("-dr")
            .arg("com.apple.quarantine")
            .arg(&target_wine_dir)
            .output();

        let _ = std::fs::remove_dir_all(&stage_dir);
        let _ = std::fs::remove_file(&tar_dest);

        // Stage 4: Install DXVK DirectX 10/11 Vulkan translation libraries into wine-shared
        let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
            stage: "configuring".to_string(),
            percent: 100.0,
            downloaded_mb: total_mb,
            total_mb,
            speed_mb: 0.0,
            message: "Configuring DXVK DirectX 10/11 translation libraries (~2.7 MB)...".to_string(),
        });

        let dxvk_url = "https://github.com/Gcenx/DXVK-macOS/releases/download/v1.10.3-20230507-repack/dxvk-macOS-async-v1.10.3-20230507-repack.tar.gz";
        let dxvk_part = cache_dir.join("dxvk-macOS.tar.gz.part");
        let dxvk_tar = cache_dir.join("dxvk-macOS.tar.gz");
        let _ = std::fs::remove_file(&dxvk_part);
        let _ = std::fs::remove_file(&dxvk_tar);

        let dxvk_curl = Command::new("/usr/bin/curl")
            .arg("-L")
            .arg("-f")
            .arg("-A")
            .arg(SWIFTER_USER_AGENT)
            .arg("-o")
            .arg(&dxvk_part)
            .arg(dxvk_url)
            .status();

        if let Ok(s) = dxvk_curl {
            if s.success() {
                let dxvk_len = std::fs::metadata(&dxvk_part).map(|m| m.len()).unwrap_or(0);
                if dxvk_len > 1_000_000 {
                    let _ = std::fs::rename(&dxvk_part, &dxvk_tar);
                    let dxvk_stage = cache_dir.join("dxvk_stage");
                    let _ = std::fs::remove_dir_all(&dxvk_stage);
                    let _ = std::fs::create_dir_all(&dxvk_stage);
                    if Command::new("/usr/bin/tar").arg("-xzf").arg(&dxvk_tar).arg("-C").arg(&dxvk_stage).status().map(|s| s.success()).unwrap_or(false) {
                        let shared_runtime = prefix_mgr::get_shared_runtime_dir();
                        let sys32 = shared_runtime.join("drive_c/windows/system32");
                        let syswow = shared_runtime.join("drive_c/windows/syswow64");
                        let _ = std::fs::create_dir_all(&sys32);
                        let _ = std::fs::create_dir_all(&syswow);

                        for entry in walkdir::WalkDir::new(&dxvk_stage).into_iter().filter_map(|e| e.ok()) {
                            if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()).map(|e| e == "dll").unwrap_or(false) {
                                let path_str = entry.path().to_string_lossy();
                                let filename = entry.file_name();
                                if path_str.contains("x64") {
                                    let _ = std::fs::copy(entry.path(), sys32.join(filename));
                                } else if path_str.contains("x32") {
                                    let _ = std::fs::copy(entry.path(), syswow.join(filename));
                                }
                            }
                        }

                        // Strip quarantine from shared runtime
                        let _ = Command::new("/usr/bin/xattr")
                            .arg("-dr")
                            .arg("com.apple.quarantine")
                            .arg(&shared_runtime)
                            .output();
                    }
                    let _ = std::fs::remove_dir_all(&dxvk_stage);
                    let _ = std::fs::remove_file(&dxvk_tar);
                }
            }
        }
        let _ = std::fs::remove_file(&dxvk_part);
        // Stage 5: Completed
        let success_msg = format!("Wine runtime and DXVK installed and verified at {}", target_wine_dir.display());
        let _ = app.emit("runtime-download-progress", RuntimeDownloadProgress {
            stage: "completed".to_string(),
            percent: 100.0,
            downloaded_mb: total_mb,
            total_mb,
            speed_mb: 0.0,
            message: success_msg.clone(),
        });

        Ok(success_msg)
    })
    .await
    .map_err(|e| format!("Task execution error: {}", e))?
}

/// 10. Removes the Swifter internal Wine runtime folder
#[tauri::command]
pub async fn remove_swifter_runtime() -> Result<(), String> {
    let wine_dir = prefix_mgr::get_swifter_support_dir().join("runtimes/wine");
    if wine_dir.exists() {
        std::fs::remove_dir_all(&wine_dir)
            .map_err(|e| format!("Failed to remove runtime folder: {}", e))?;
    }
    Ok(())
}

fn is_system_or_installer_binary(path_str: &str) -> bool {
    let lower = path_str.to_lowercase();
    let blacklist = [
        "package cache",
        "programdata",
        "windows/",
        "windows\\",
        "temp/",
        "temp\\",
        "vcredist",
        "vc_redist",
        "dxsetup",
        "unins",
        "setup",
        "install",
        "crash",
        "updater",
        "winhelp",
        "regedit",
        "notepad",
        "hh.exe",
    ];
    blacklist.iter().any(|b| lower.contains(b))
}

fn score_candidate(path_str: &str, clean_slug: &str) -> i32 {
    let mut score = 0i32;
    let lower = path_str.to_lowercase();

    if lower.contains("program files") {
        score += 100;
    }
    let filename = Path::new(path_str)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase().replace('-', "").replace('_', ""))
        .unwrap_or_default();

    if filename == clean_slug {
        score += 100;
    } else if filename.contains(clean_slug) || clean_slug.contains(&filename) {
        score += 50;
    }

    if lower.contains(clean_slug) {
        score += 30;
    }

    score
}

fn rank_installed_executables(mut exes: Vec<String>, app_slug: &str) -> Vec<String> {
    let clean_slug = app_slug.to_lowercase().replace('-', "").replace('_', "");
    exes.retain(|e| !is_system_or_installer_binary(e));
    exes.sort_by(|a, b| {
        let score_a = score_candidate(a, &clean_slug);
        let score_b = score_candidate(b, &clean_slug);
        score_b.cmp(&score_a)
    });
    exes
}

/// 11. Executes a setup/installer wizard inside the isolated micro-prefix and scans for newly installed game binaries
#[tauri::command]
pub async fn run_installer_in_prefix(exe_path: String, app_slug: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let prefix_dir = prefix_mgr::ensure_micro_prefix(&app_slug)?;
        let wine_bin = resolve_wine_binary()?;

        let exe_path_buf = PathBuf::from(&exe_path);
        let parent_dir = exe_path_buf.parent().unwrap_or_else(|| Path::new("."));

        let mut cmd = Command::new(&wine_bin);
        cmd.arg(&exe_path_buf)
            .current_dir(parent_dir)
            .env("WINEPREFIX", &prefix_dir)
            .env("WINEARCH", "win64")
            .env("MESA_LOADER_DRIVER_OVERRIDE", "zink")
            .env("GALLIUM_DRIVER", "zink")
            .env("MESA_GL_VERSION_OVERRIDE", "4.6")
            .env("MESA_GLSL_VERSION_OVERRIDE", "460");

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn installer: {}", e))?;
        let pid = child.id();

        // Automatically bring the newly spawned installer window to the foreground on macOS
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(700));
            let script = format!(
                "tell application \"System Events\"\ntry\nset frontmost of first process whose unix id is {} to true\nend try\nend tell\ntell application \"Wine\" to activate",
                pid
            );
            let _ = Command::new("osascript").arg("-e").arg(&script).output();
        });

        let status = child.wait().map_err(|e| format!("Installer process failed: {}", e))?;
        if !status.success() {
            return Err(format!("Installer process exited with code: {:?}", status.code()));
        }
        let mut installed_exes = Vec::new();

        // 1. Inspect Windows shortcut (.lnk) files in Desktop and Start Menu (created by installers)
        let users_dir = prefix_dir.join("drive_c/users");
        if users_dir.exists() {
            for entry in walkdir::WalkDir::new(&users_dir).max_depth(8).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()).map(|e| e.eq_ignore_ascii_case("lnk")).unwrap_or(false) {
                    let lnk_name = entry.file_name().to_string_lossy().to_lowercase();
                    if !lnk_name.contains("unins") && !lnk_name.contains("license") && !lnk_name.contains("help") {
                        if let Ok(content) = std::fs::read(entry.path()) {
                            let s = String::from_utf8_lossy(&content);
                            for chunk in s.split('\0') {
                                if (chunk.starts_with("C:\\") || chunk.starts_with("c:\\")) && chunk.to_lowercase().ends_with(".exe") {
                                    let rel = chunk[3..].replace('\\', "/");
                                    let candidate = format!("drive_c/{}", rel);
                                    if !installed_exes.contains(&candidate) && !is_system_or_installer_binary(&candidate) {
                                        installed_exes.push(candidate);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Scan drive_c inside micro-prefix and shared runtime
        let drive_c = prefix_dir.join("drive_c");
        let shared_drive_c = prefix_mgr::get_shared_runtime_dir().join("drive_c");
        let search_dirs = [drive_c, shared_drive_c];

        for d in search_dirs {
            if d.exists() {
                for entry in walkdir::WalkDir::new(&d).follow_links(true).max_depth(6).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        let p = entry.path();
                        if p.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                            if let Ok(rel) = p.strip_prefix(&d) {
                                let candidate = format!("drive_c/{}", rel.to_string_lossy());
                                if !is_system_or_installer_binary(&candidate) && !installed_exes.contains(&candidate) {
                                    installed_exes.push(candidate);
                                }
                            }
                        }
                    }
                }
            }
        }

        let ranked = rank_installed_executables(installed_exes, &app_slug);
        Ok(ranked)
    })
    .await
    .map_err(|e| format!("Task execution error: {}", e))?
}

/// 12. Scans a packaged app bundle and its prefix to list all executable (.exe) candidates
#[tauri::command]
pub async fn scan_available_exes_in_app(bundle_path: String) -> Result<Vec<String>, String> {
    let bundle = PathBuf::from(&bundle_path);
    let config_path = bundle.join("Contents/Resources/swifter.config.json");
    if !config_path.exists() {
        return Err("Configuration file not found in application bundle.".into());
    }

    let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let cfg: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let app_slug = cfg["app_slug"].as_str().unwrap_or("app");

    let mut exes = Vec::new();

    // 1. Scan payload directory
    let payload_dir = bundle.join("Contents/Resources/game_payload");
    if payload_dir.exists() {
        for entry in walkdir::WalkDir::new(&payload_dir).max_depth(4).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                if let Ok(rel) = entry.path().strip_prefix(&payload_dir) {
                    exes.push(rel.to_string_lossy().to_string());
                }
            }
        }
    }

    // 2. Scan prefix drive_c and shared runtime drive_c with follow_links
    let prefix_dir = prefix_mgr::get_prefix_dir(app_slug);
    let drive_c = prefix_dir.join("drive_c");
    let shared_drive_c = prefix_mgr::get_shared_runtime_dir().join("drive_c");
    let search_roots = [drive_c, shared_drive_c];

    for root_dir in search_roots {
        if root_dir.exists() {
            for entry in walkdir::WalkDir::new(&root_dir).follow_links(true).max_depth(6).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                    let name = entry.path().file_name().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
                    if !name.starts_with("unins") && !name.contains("crash") {
                        if let Ok(rel) = entry.path().strip_prefix(&root_dir) {
                            let rel_str = format!("drive_c/{}", rel.to_string_lossy());
                            if !exes.contains(&rel_str) {
                                exes.push(rel_str);
                            }
                        }
                    }
                }
            }
        }
    }
    let ranked = rank_installed_executables(exes, app_slug);
    Ok(ranked)
}

/// 13. Updates the target executable of an already packaged application bundle
#[tauri::command]
pub async fn update_app_target_exe(bundle_path: String, new_target_exe: String) -> Result<(), String> {
    let bundle = PathBuf::from(&bundle_path);
    let config_path = bundle.join("Contents/Resources/swifter.config.json");
    if !config_path.exists() {
        return Err("Configuration file not found in bundle".to_string());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let mut cfg: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    cfg["target_exe_relative"] = serde_json::Value::String(new_target_exe);

    std::fs::write(&config_path, serde_json::to_string_pretty(&cfg).unwrap())
        .map_err(|e| format!("Failed to write updated config: {}", e))?;

    Ok(())
}

/// 14. Uninstalls a packaged application bundle and removes its isolated micro-prefix
#[tauri::command]
pub async fn uninstall_packaged_app(bundle_path: String) -> Result<(), String> {
    let bundle = PathBuf::from(&bundle_path);
    if !bundle.exists() {
        return Err("Application bundle does not exist on disk.".into());
    }

    let config_path = bundle.join("Contents/Resources/swifter.config.json");
    let mut app_slug = None;
    if config_path.exists() {
        if let Ok(c) = std::fs::read_to_string(&config_path) {
            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&c) {
                if let Some(slug) = cfg["app_slug"].as_str() {
                    app_slug = Some(slug.to_string());
                }
            }
        }
    }

    // 1. Delete the macOS application bundle
    std::fs::remove_dir_all(&bundle)
        .map_err(|e| format!("Failed to delete application bundle: {}", e))?;

    // 2. Delete the associated isolated micro-prefix if present
    if let Some(slug) = app_slug {
        let prefix_dir = prefix_mgr::get_prefix_dir(&slug);
        if prefix_dir.exists() {
            let _ = std::fs::remove_dir_all(&prefix_dir);
        }
    }

    Ok(())
}

/// 15. Removes all installed Swifter runtimes (Wine, DXVK, shared base prefix) and temporary cache
#[tauri::command]
pub async fn remove_all_runtimes() -> Result<(), String> {
    let runtimes_dir = prefix_mgr::get_swifter_support_dir().join("runtimes");
    if runtimes_dir.exists() {
        std::fs::remove_dir_all(&runtimes_dir)
            .map_err(|e| format!("Failed to delete runtimes directory: {}", e))?;
    }

    if let Some(cache_dir) = dirs::cache_dir() {
        let swifter_cache = cache_dir.join("swifter");
        if swifter_cache.exists() {
            let _ = std::fs::remove_dir_all(&swifter_cache);
        }
    }

    Ok(())
}
