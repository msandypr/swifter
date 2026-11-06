use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

/// Returns the root Application Support directory for Swifter:
/// ~/Library/Application Support/Swifter
pub fn get_swifter_support_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("Library/Application Support"))
        .join("Swifter")
}

/// Returns the shared base Wine runtime directory:
/// ~/Library/Application Support/Swifter/runtimes/wine-shared
pub fn get_shared_runtime_dir() -> PathBuf {
    get_swifter_support_dir().join("runtimes").join("wine-shared")
}

/// Returns the isolated prefix directory for a specific app:
/// ~/Library/Application Support/Swifter/prefixes/<app-slug>
pub fn get_prefix_dir(app_slug: &str) -> PathBuf {
    get_swifter_support_dir().join("prefixes").join(app_slug)
}

/// Returns the isolated user AppData directory for save games:
/// ~/Library/Application Support/Swifter/prefixes/<app-slug>/drive_c/users/<username>/AppData
pub fn get_save_data_dir(app_slug: &str) -> PathBuf {
    let username = whoami_user();
    get_prefix_dir(app_slug)
        .join("drive_c")
        .join("users")
        .join(&username)
        .join("AppData")
}

fn whoami_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "user".to_string())
}

/// Initializes an isolated micro-prefix for an application.
/// Uses symlinks to the shared base runtime for windows/system32/syswow64/Program Files,
/// but creates isolated AppData, Saved Games, and registry files for the app (~15-30MB).
pub fn ensure_micro_prefix(app_slug: &str) -> Result<PathBuf, String> {
    let shared_runtime = get_shared_runtime_dir();
    let prefix_dir = get_prefix_dir(app_slug);
    let username = whoami_user();

    // Ensure shared base runtime structure exists
    ensure_shared_base_skeleton(&shared_runtime)?;

    // Create prefix root and drive_c
    let drive_c = prefix_dir.join("drive_c");
    let windows_dir = drive_c.join("windows");
    let users_dir = drive_c.join("users").join(&username);
    let dosdevices = prefix_dir.join("dosdevices");

    fs::create_dir_all(&windows_dir)
        .map_err(|e| format!("Failed to create windows dir in prefix: {}", e))?;
    fs::create_dir_all(&users_dir.join("AppData").join("Local"))
        .map_err(|e| format!("Failed to create AppData/Local in prefix: {}", e))?;
    fs::create_dir_all(&users_dir.join("AppData").join("LocalLow"))
        .map_err(|e| format!("Failed to create AppData/LocalLow in prefix: {}", e))?;
    fs::create_dir_all(&users_dir.join("AppData").join("Roaming"))
        .map_err(|e| format!("Failed to create AppData/Roaming in prefix: {}", e))?;
    fs::create_dir_all(&users_dir.join("Saved Games"))
        .map_err(|e| format!("Failed to create Saved Games in prefix: {}", e))?;
    fs::create_dir_all(&users_dir.join("Documents"))
        .map_err(|e| format!("Failed to create Documents in prefix: {}", e))?;
    fs::create_dir_all(&dosdevices)
        .map_err(|e| format!("Failed to create dosdevices in prefix: {}", e))?;

    // Create dosdevices symlinks (c: -> ../drive_c, z: -> /)
    safe_symlink(&PathBuf::from("../drive_c"), &dosdevices.join("c:"));
    safe_symlink(&PathBuf::from("/"), &dosdevices.join("z:"));

    // Symlink shared windows subdirectories (system32, syswow64, fonts)
    safe_symlink(&shared_runtime.join("drive_c/windows/system32"), &windows_dir.join("system32"));
    safe_symlink(&shared_runtime.join("drive_c/windows/syswow64"), &windows_dir.join("syswow64"));
    safe_symlink(&shared_runtime.join("drive_c/windows/fonts"), &windows_dir.join("fonts"));

    // Symlink Program Files
    safe_symlink(&shared_runtime.join("drive_c/Program Files"), &drive_c.join("Program Files"));
    safe_symlink(&shared_runtime.join("drive_c/Program Files (x86)"), &drive_c.join("Program Files (x86)"));

    // Registry files: symlink system.reg to shared, keep user.reg and userdef.reg isolated
    safe_symlink(&shared_runtime.join("system.reg"), &prefix_dir.join("system.reg"));

    let user_reg = prefix_dir.join("user.reg");
    if !user_reg.exists() {
        let minimal_user_reg = "WINE REGISTRY Version 2\n;; All keys relative to \\\\User\n\n#arch=win64\n\n[Software\\\\Wine\\\\Direct3D]\n\"MaxVersionGL\"=dword:00040006\n\n[Software\\\\Wine\\\\DllOverrides]\n\"opengl32\"=\"native\"\n\n";
        let _ = fs::write(&user_reg, minimal_user_reg);
    } else if let Ok(c) = fs::read_to_string(&user_reg) {
        if c.contains("#arch=win32") {
            let _ = fs::write(&user_reg, c.replace("#arch=win32", "#arch=win64"));
        }
    }

    let userdef_reg = prefix_dir.join("userdef.reg");
    if !userdef_reg.exists() {
        let minimal_userdef_reg = "WINE REGISTRY Version 2\n;; All keys relative to \\\\UserDef\n\n#arch=win64\n\n";
        let _ = fs::write(&userdef_reg, minimal_userdef_reg);
    } else if let Ok(c) = fs::read_to_string(&userdef_reg) {
        if c.contains("#arch=win32") {
            let _ = fs::write(&userdef_reg, c.replace("#arch=win32", "#arch=win64"));
        }
    }

    Ok(prefix_dir)
}

/// Ensures the minimal skeleton of the shared base runtime exists
fn ensure_shared_base_skeleton(shared_runtime: &Path) -> Result<(), String> {
    let drive_c = shared_runtime.join("drive_c");
    let windows = drive_c.join("windows");

    fs::create_dir_all(windows.join("system32"))
        .map_err(|e| format!("Failed to init shared system32: {}", e))?;
    fs::create_dir_all(windows.join("syswow64"))
        .map_err(|e| format!("Failed to init shared syswow64: {}", e))?;
    fs::create_dir_all(windows.join("fonts"))
        .map_err(|e| format!("Failed to init shared fonts: {}", e))?;
    fs::create_dir_all(drive_c.join("Program Files"))
        .map_err(|e| format!("Failed to init shared Program Files: {}", e))?;
    fs::create_dir_all(drive_c.join("Program Files (x86)"))
        .map_err(|e| format!("Failed to init shared Program Files (x86): {}", e))?;

    let system_reg = shared_runtime.join("system.reg");
    if !system_reg.exists() {
        let minimal_system_reg = "WINE REGISTRY Version 2\n;; All keys relative to \\\\System\n\n#arch=win64\n\n";
        let _ = fs::write(&system_reg, minimal_system_reg);
    } else if let Ok(c) = fs::read_to_string(&system_reg) {
        if c.contains("#arch=win32") {
            let _ = fs::write(&system_reg, c.replace("#arch=win32", "#arch=win64"));
        }
    }

    // Automatically sync system TrueType fonts into shared fonts directory
    let _ = sync_macos_fonts_to_shared_prefix();
    Ok(())
}

/// Discovers TrueType and OpenType fonts on macOS and links/copies them to wine-shared fonts
pub fn sync_macos_fonts_to_shared_prefix() -> Result<usize, String> {
    let shared_fonts = get_shared_runtime_dir().join("drive_c/windows/fonts");
    let _ = fs::create_dir_all(&shared_fonts);

    let sources = [
        PathBuf::from("/System/Library/Fonts/Supplemental"),
        PathBuf::from("/Library/Fonts"),
    ];

    let mut count = 0;
    for src in sources {
        if src.exists() {
            if let Ok(entries) = fs::read_dir(src) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf") {
                            let file_name = entry.file_name();
                            let target = shared_fonts.join(&file_name);
                            if !target.exists() {
                                if symlink(&path, &target).is_err() {
                                    let _ = fs::copy(&path, &target);
                                }
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(count)
}

fn safe_symlink(target: &Path, link_path: &Path) {
    if link_path.exists() || link_path.is_symlink() {
        let _ = fs::remove_file(link_path);
        let _ = fs::remove_dir(link_path);
    }
    let _ = symlink(target, link_path);
}

/// Updates or inserts a registry key in a Wine .reg file string
pub fn update_or_insert_reg_key(content: &str, section_pattern: &str, key_name: &str, val_str: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut in_section = false;
    let mut section_found = false;
    let mut key_updated = false;

    let clean_pat = section_pattern.replace("\\\\", "\\");

    for i in 0..lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let sec_name = &trimmed[1..trimmed.len() - 1];
            let clean_sec = sec_name.replace("\\\\", "\\");

            if clean_sec.eq_ignore_ascii_case(&clean_pat) {
                in_section = true;
                section_found = true;
                continue;
            } else if in_section {
                lines.insert(i, format!("\"{}\"={}", key_name, val_str));
                key_updated = true;
                break;
            }
        }

        if in_section {
            let key_prefix = format!("\"{}\"=", key_name);
            if trimmed.starts_with(&key_prefix) {
                lines[i] = format!("\"{}\"={}", key_name, val_str);
                key_updated = true;
                break;
            }
        }
    }

    if section_found && !key_updated {
        lines.push(format!("\"{}\"={}", key_name, val_str));
    } else if !section_found {
        lines.push(String::new());
        lines.push(format!("[{}]", section_pattern));
        lines.push(format!("\"{}\"={}", key_name, val_str));
    }

    lines.join("\n") + "\n"
}

/// Applies the specified DPI (e.g. 96, 120, 144, 192) and Retina mode to user.reg and userdef.reg
pub fn apply_dpi_and_retina_settings(prefix_dir: &Path, dpi: u32, enable_retina: bool) -> Result<(), String> {
    let user_reg = prefix_dir.join("user.reg");
    let dpi_val = if dpi == 0 { 96 } else { dpi };
    let hex_dpi = format!("{:08x}", dpi_val);
    let retina_str = if enable_retina { "\"Y\"" } else { "\"N\"" };

    let mut content = if user_reg.exists() {
        fs::read_to_string(&user_reg).unwrap_or_default()
    } else {
        "WINE REGISTRY Version 2\n;; All keys relative to \\\\User\n\n#arch=win64\n\n".to_string()
    };

    content = update_or_insert_reg_key(
        &content,
        "Control Panel\\Desktop",
        "LogPixels",
        &format!("dword:{}", hex_dpi),
    );

    content = update_or_insert_reg_key(
        &content,
        "Software\\Wine\\Mac Driver",
        "RetinaMode",
        retina_str,
    );

    fs::write(&user_reg, content).map_err(|e| format!("Failed to write user.reg: {}", e))?;

    let userdef_reg = prefix_dir.join("userdef.reg");
    if userdef_reg.exists() {
        if let Ok(c) = fs::read_to_string(&userdef_reg) {
            let u1 = update_or_insert_reg_key(
                &c,
                "Control Panel\\Desktop",
                "LogPixels",
                &format!("dword:{}", hex_dpi),
            );
            let u2 = update_or_insert_reg_key(
                &u1,
                "Software\\Wine\\Mac Driver",
                "RetinaMode",
                retina_str,
            );
            let _ = fs::write(&userdef_reg, u2);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_or_insert_reg_key_new_section() {
        let initial = "WINE REGISTRY Version 2\n#arch=win64\n";
        let res = update_or_insert_reg_key(initial, "Control Panel\\Desktop", "LogPixels", "dword:00000090");
        assert!(res.contains("[Control Panel\\Desktop]"));
        assert!(res.contains("\"LogPixels\"=dword:00000090"));
    }

    #[test]
    fn test_update_or_insert_reg_key_existing_key() {
        let initial = "WINE REGISTRY Version 2\n\n[Control Panel\\Desktop]\n\"LogPixels\"=dword:00000060\n\"Wallpaper\"=\"\"\n";
        let res = update_or_insert_reg_key(initial, "Control Panel\\Desktop", "LogPixels", "dword:000000c0");
        assert!(res.contains("\"LogPixels\"=dword:000000c0"));
        assert!(!res.contains("dword:00000060"));
        assert!(res.contains("\"Wallpaper\"=\"\""));
    }

    #[test]
    fn test_apply_dpi_and_retina_settings() {
        let temp_dir = std::env::temp_dir().join("swifter_test_dpi_prefix");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        apply_dpi_and_retina_settings(&temp_dir, 144, true).unwrap();

        let user_reg_content = fs::read_to_string(temp_dir.join("user.reg")).unwrap();
        assert!(user_reg_content.contains("\"LogPixels\"=dword:00000090"));
        assert!(user_reg_content.contains("\"RetinaMode\"=\"Y\""));

        // Change to 96 and false
        apply_dpi_and_retina_settings(&temp_dir, 96, false).unwrap();
        let updated = fs::read_to_string(temp_dir.join("user.reg")).unwrap();
        assert!(updated.contains("\"LogPixels\"=dword:00000060"));
        assert!(updated.contains("\"RetinaMode\"=\"N\""));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
