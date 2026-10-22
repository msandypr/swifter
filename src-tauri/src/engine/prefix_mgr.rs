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
        let minimal_user_reg = "WINE REGISTRY Version 2\n;; All keys relative to \\\\User\n\n#arch=win64\n\n[Software\\\\Wine\\\\Direct3D]\n\"MaxVersionGL\"=dword:00040006\n\n";
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
    Ok(())
}

fn safe_symlink(target: &Path, link_path: &Path) {
    if link_path.exists() || link_path.is_symlink() {
        let _ = fs::remove_file(link_path);
        let _ = fs::remove_dir(link_path);
    }
    let _ = symlink(target, link_path);
}
