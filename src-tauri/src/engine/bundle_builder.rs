use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use slugify::slugify;

use super::launcher_shim::{self, SwifterAppConfig};
use super::prefix_mgr;
use crate::analyzer::icon_extract;

pub struct BuildAppOptions {
    pub game_folder_path: PathBuf,
    pub target_exe_relative: String,
    pub app_name: String,
    pub backend_choice: String,
    pub enable_msync: bool,
    pub enable_esync: bool,
    pub enable_retina: bool,
    pub copy_files_instead_of_symlink: bool,
    pub output_dir: Option<PathBuf>,
    pub cached_icns_path: Option<PathBuf>,
}

/// Assembles a native macOS .app bundle
pub fn build_application_bundle(options: BuildAppOptions) -> Result<PathBuf, String> {
    if !options.game_folder_path.exists() {
        return Err(format!(
            "Game folder does not exist: {}",
            options.game_folder_path.display()
        ));
    }

    let clean_app_name = options.app_name.trim();
    if clean_app_name.is_empty() {
        return Err("Application name cannot be empty".into());
    }

    let mut app_slug = slugify(clean_app_name, "", "-", None);
    if app_slug.is_empty() {
        app_slug = "app".to_string();
    }
    let app_bundle_name = format!("{}.app", clean_app_name);

    // Determine target output directory
    let output_parent = match options.output_dir {
        Some(dir) => dir,
        None => get_default_applications_dir()?,
    };

    let bundle_path = output_parent.join(&app_bundle_name);

    // If an existing bundle with the same name exists, safely remove it
    if bundle_path.exists() {
        let _ = fs::remove_dir_all(&bundle_path);
    }

    let contents_dir = bundle_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");

    fs::create_dir_all(&macos_dir)
        .map_err(|e| format!("Failed to create bundle MacOS directory: {}", e))?;
    fs::create_dir_all(&resources_dir)
        .map_err(|e| format!("Failed to create bundle Resources directory: {}", e))?;

    // 1. Create Info.plist
    let info_plist = generate_info_plist(&app_slug, clean_app_name);
    fs::write(contents_dir.join("Info.plist"), info_plist)
        .map_err(|e| format!("Failed to write Info.plist: {}", e))?;

    // 2. Create PkgInfo
    fs::write(contents_dir.join("PkgInfo"), "APPL????")
        .map_err(|e| format!("Failed to write PkgInfo: {}", e))?;

    // 3. Create swifter.config.json
    let config = SwifterAppConfig {
        app_name: clean_app_name.to_string(),
        app_slug: app_slug.clone(),
        target_exe_relative: options.target_exe_relative.clone(),
        backend: options.backend_choice.clone(),
        enable_msync: options.enable_msync,
        enable_esync: options.enable_esync,
        enable_retina: options.enable_retina,
        custom_env: std::collections::HashMap::new(),
        wine_custom_path: None,
    };

    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize swifter config: {}", e))?;
    fs::write(resources_dir.join("swifter.config.json"), config_json)
        .map_err(|e| format!("Failed to write swifter.config.json: {}", e))?;

    // 4. Create launcher executable with chmod +x
    let launcher_script = launcher_shim::generate_launcher_script(&config);
    let launcher_path = macos_dir.join("launcher");
    fs::write(&launcher_path, launcher_script)
        .map_err(|e| format!("Failed to write launcher script: {}", e))?;

    let permissions = fs::Permissions::from_mode(0o755);
    fs::set_permissions(&launcher_path, permissions)
        .map_err(|e| format!("Failed to set launcher executable permissions: {}", e))?;

    // 5. Place AppIcon.icns
    let target_icns = resources_dir.join("AppIcon.icns");
    let mut icon_copied = false;

    if let Some(cached_icns) = &options.cached_icns_path {
        if cached_icns.exists() {
            if fs::copy(cached_icns, &target_icns).is_ok() {
                icon_copied = true;
            }
        }
    }

    if !icon_copied {
        let prefix_dir = prefix_mgr::get_prefix_dir(&app_slug);
        let shared_runtime = prefix_mgr::get_shared_runtime_dir();

        let mut possible_exes = Vec::new();
        possible_exes.push(options.game_folder_path.join(&options.target_exe_relative));
        possible_exes.push(prefix_dir.join(&options.target_exe_relative));
        possible_exes.push(prefix_dir.join("drive_c").join(&options.target_exe_relative));
        possible_exes.push(shared_runtime.join(&options.target_exe_relative));
        possible_exes.push(shared_runtime.join("drive_c").join(&options.target_exe_relative));

        if let Some(stripped) = options.target_exe_relative.strip_prefix("drive_c/") {
            possible_exes.push(prefix_dir.join("drive_c").join(stripped));
            possible_exes.push(shared_runtime.join("drive_c").join(stripped));
        }

        // Also check any .exe directly in game_folder_path
        if let Ok(entries) = fs::read_dir(&options.game_folder_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().extension().and_then(|s| s.to_str()).map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                    possible_exes.push(entry.path());
                }
            }
        }

        for exe in possible_exes {
            if exe.exists() {
                if let Ok(icns) = icon_extract::extract_and_generate_icns(&exe) {
                    if icns.exists() {
                        if fs::copy(&icns, &target_icns).is_ok() {
                            icon_copied = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    if !icon_copied {
        // Generate stylized fallback icon (never a blank image)
        let fallback_img = icon_extract::generate_fallback_game_icon();
        let temp_fallback = std::env::temp_dir().join(format!("{}_fallback.icns", app_slug));
        if icon_extract::pack_image_to_icns(&fallback_img, &temp_fallback).is_ok() {
            let _ = fs::copy(&temp_fallback, &target_icns);
        }
    }

    // 6. Connect game payload (symlink or full copy)
    let payload_path = resources_dir.join("game_payload");
    if options.copy_files_instead_of_symlink {
        copy_dir_recursive(&options.game_folder_path, &payload_path)
            .map_err(|e| format!("Failed to copy game payload: {}", e))?;
    } else {
        symlink(&options.game_folder_path, &payload_path)
            .map_err(|e| format!("Failed to symlink game payload: {}", e))?;
    }

    // 7. Ensure isolated micro-prefix is provisioned
    prefix_mgr::ensure_micro_prefix(&app_slug)?;

    Ok(bundle_path)
}

fn get_default_applications_dir() -> Result<PathBuf, String> {
    let main_apps = PathBuf::from("/Applications");
    // Check if /Applications is writable
    if let Ok(metadata) = fs::metadata(&main_apps) {
        if !metadata.permissions().readonly() {
            let test_file = main_apps.join(".swifter_write_test");
            if fs::write(&test_file, b"").is_ok() {
                let _ = fs::remove_file(&test_file);
                return Ok(main_apps);
            }
        }
    }

    // Fallback to ~/Applications
    let user_apps = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Applications");
    fs::create_dir_all(&user_apps)
        .map_err(|e| format!("Failed to create ~/Applications: {}", e))?;
    Ok(user_apps)
}

fn generate_info_plist(app_slug: &str, app_name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>launcher</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.swifter.app.{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
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
"#,
        app_slug, app_name
    )
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let rel_path = entry.path().strip_prefix(src).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
        let target_path = dst.join(rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path)?;
        } else if entry.file_type().is_symlink() {
            if let Ok(link_target) = fs::read_link(entry.path()) {
                let _ = symlink(link_target, &target_path);
            }
        } else {
            fs::copy(entry.path(), &target_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_plist_generation() {
        let plist = generate_info_plist("my-cool-game", "My Cool Game");
        assert!(plist.contains("<string>com.swifter.app.my-cool-game</string>"));
        assert!(plist.contains("<string>My Cool Game</string>"));
        assert!(plist.contains("<string>launcher</string>"));
        assert!(plist.contains("<string>AppIcon</string>"));
    }

    #[test]
    fn test_bundle_assembly_mock() {
        let temp_dir = std::env::temp_dir().join("swifter_test_bundle_root");
        let _ = fs::remove_dir_all(&temp_dir);
        let game_folder = temp_dir.join("MockGame");
        fs::create_dir_all(&game_folder).unwrap();
        fs::write(game_folder.join("game.exe"), b"MOCK_EXE_DATA").unwrap();

        let out_dir = temp_dir.join("OutputApps");
        fs::create_dir_all(&out_dir).unwrap();

        let options = BuildAppOptions {
            game_folder_path: game_folder,
            target_exe_relative: "game.exe".to_string(),
            app_name: "Mock Indie Game".to_string(),
            backend_choice: "DXVK".to_string(),
            enable_msync: true,
            enable_esync: true,
            enable_retina: false,
            copy_files_instead_of_symlink: false,
            output_dir: Some(out_dir.clone()),
            cached_icns_path: None,
        };

        let bundle_res = build_application_bundle(options);
        assert!(bundle_res.is_ok(), "Bundle build failed: {:?}", bundle_res);
        let bundle = bundle_res.unwrap();
        assert!(bundle.exists());
        assert!(bundle.join("Contents/Info.plist").exists());
        assert!(bundle.join("Contents/PkgInfo").exists());
        assert!(bundle.join("Contents/Resources/swifter.config.json").exists());
        assert!(bundle.join("Contents/Resources/game_payload").exists());

        let launcher = bundle.join("Contents/MacOS/launcher");
        assert!(launcher.exists());
        let perm = fs::metadata(&launcher).unwrap().permissions();
        assert_eq!(perm.mode() & 0o111, 0o111, "Launcher must be executable");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
