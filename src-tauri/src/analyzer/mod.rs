pub mod folder_scan;
pub mod icon_extract;
pub mod pe_parser;

use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SwifterInspection {
    pub target_exe_path: String,
    pub target_exe_name: String,
    pub game_folder_path: String,
    pub app_display_name: String,
    pub is_64bit: bool,
    pub detected_engine: String,
    pub recommended_backend: String,
    pub is_anticheat_blocked: bool,
    pub anticheat_reason: Option<String>,
    pub icon_cached_path: Option<String>,
    pub exe_candidates: Vec<String>,
    pub is_installer: bool,
}

/// Comprehensive inspection function for a dropped file or folder
pub fn inspect_target_path(dropped_path_str: &str) -> Result<SwifterInspection, String> {
    let dropped_path = Path::new(dropped_path_str);
    if !dropped_path.exists() {
        return Err(format!("Path does not exist: {}", dropped_path_str));
    }

    // 1. Scan folder context and resolve target executable
    let scan = folder_scan::scan_game_context(dropped_path)?;

    // 2. Parse PE binary header (architecture + version metadata)
    let pe_meta = pe_parser::parse_pe_binary(&scan.target_exe)?;

    // 3. Check anti-cheat guard across the entire game directory
    let anticheat_reason = pe_parser::check_anticheat_guard(&scan.game_root);
    let is_anticheat_blocked = anticheat_reason.is_some();

    // 4. Derive best display name for the application
    let raw_stem = scan
        .target_exe
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Windows Application".into());

    let app_display_name = pe_meta
        .product_name
        .clone()
        .or_else(|| pe_meta.file_description.clone())
        .unwrap_or(raw_stem);

    let target_exe_name = scan
        .target_exe
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // 5. Extract icon and convert to Apple .icns
    let icon_cached_path = icon_extract::extract_and_generate_icns(&scan.target_exe)
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let candidates = scan
        .exe_candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let detected_engine_str = scan.engine.as_str().to_string();
    let recommended_backend = scan.engine.default_recommended_backend().to_string();
    let is_installer = {
        let lower_name = scan
            .target_exe
            .file_name()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let name_match = lower_name.starts_with("setup")
            || lower_name.starts_with("install")
            || lower_name.contains("installer")
            || lower_name.contains("setup");
        let meta_match = pe_meta
            .file_description
            .as_ref()
            .map(|d| d.to_lowercase().contains("setup") || d.to_lowercase().contains("installer"))
            .unwrap_or(false)
            || pe_meta
                .product_name
                .as_ref()
                .map(|p| p.to_lowercase().contains("setup") || p.to_lowercase().contains("installer"))
                .unwrap_or(false);
        name_match || meta_match
    };

    Ok(SwifterInspection {
        target_exe_path: scan.target_exe.to_string_lossy().to_string(),
        target_exe_name,
        game_folder_path: scan.game_root.to_string_lossy().to_string(),
        app_display_name,
        is_64bit: pe_meta.is_64bit,
        detected_engine: detected_engine_str,
        recommended_backend,
        is_anticheat_blocked,
        anticheat_reason,
        icon_cached_path,
        exe_candidates: candidates,
        is_installer,
    })
}
