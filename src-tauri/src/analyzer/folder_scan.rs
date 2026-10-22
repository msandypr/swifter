use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedEngine {
    Unity,
    Godot,
    GameMaker,
    Unreal,
    Generic,
}

impl DetectedEngine {
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectedEngine::Unity => "Unity",
            DetectedEngine::Godot => "Godot",
            DetectedEngine::GameMaker => "GameMaker",
            DetectedEngine::Unreal => "Unreal",
            DetectedEngine::Generic => "Generic",
        }
    }

    pub fn default_recommended_backend(&self) -> &'static str {
        match self {
            DetectedEngine::Unity => "DXVK",
            DetectedEngine::Godot => "DXVK",
            DetectedEngine::GameMaker => "Vanilla",
            DetectedEngine::Unreal => "DXVK",
            DetectedEngine::Generic => "DXVK",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub game_root: PathBuf,
    pub target_exe: PathBuf,
    pub engine: DetectedEngine,
    pub exe_candidates: Vec<PathBuf>,
}

/// Blacklisted executable name patterns (case-insensitive)
const IGNORED_EXE_PREFIXES: &[&str] = &[
    "unins",
    "unitycrashhandler",
    "crashreport",
    "dxwebsetup",
    "dxsetup",
    "vcredist",
    "dotnetfx",
    "setup",
    "install",
];

/// Resolves dropped path (either file or folder) and analyzes folder context
pub fn scan_game_context(dropped_path: &Path) -> Result<ScanResult, String> {
    if !dropped_path.exists() {
        return Err(format!("Specified path does not exist: {}", dropped_path.display()));
    }

    let (game_root, target_exe, candidates) = if dropped_path.is_file() {
        let ext = dropped_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if ext != "exe" {
            return Err("Dropped file is not a Windows executable (.exe)".into());
        }
        let root = dropped_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let candidates = find_exe_candidates(&root);
        (root, dropped_path.to_path_buf(), candidates)
    } else if dropped_path.is_dir() {
        let candidates = find_exe_candidates(dropped_path);
        if candidates.is_empty() {
            return Err(format!(
                "No Windows executable (.exe) found in directory: {}",
                dropped_path.display()
            ));
        }

        // Pick best executable candidate based on heuristic scoring
        let best_exe = pick_primary_executable(dropped_path, &candidates);
        (dropped_path.to_path_buf(), best_exe, candidates)
    } else {
        return Err("Dropped path is neither a file nor a directory".into());
    };

    let engine = detect_game_engine(&game_root);

    Ok(ScanResult {
        game_root,
        target_exe,
        engine,
        exe_candidates: candidates,
    })
}

/// Detects indie game engine heuristics by scanning folder contents
pub fn detect_game_engine(folder: &Path) -> DetectedEngine {
    let mut has_unity_player = false;
    let mut has_unity_data_folder = false;
    let mut has_godot_pck = false;
    let mut has_gamemaker_data = false;
    let mut has_unreal_engine = false;

    for entry in WalkDir::new(folder)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_name = entry.file_name().to_string_lossy().to_lowercase();
        let path = entry.path();

        if file_name == "unityplayer.dll" {
            has_unity_player = true;
        }

        if entry.file_type().is_dir() && file_name.ends_with("_data") {
            has_unity_data_folder = true;
        }

        if file_name.ends_with(".pck") {
            has_godot_pck = true;
        }

        if file_name == "data.win" {
            has_gamemaker_data = true;
        }

        if file_name == "engine" && entry.file_type().is_dir() {
            let binaries = path.join("Binaries");
            if binaries.exists() {
                has_unreal_engine = true;
            }
        }
    }

    if has_unity_player || has_unity_data_folder {
        DetectedEngine::Unity
    } else if has_godot_pck {
        DetectedEngine::Godot
    } else if has_gamemaker_data {
        DetectedEngine::GameMaker
    } else if has_unreal_engine {
        DetectedEngine::Unreal
    } else {
        DetectedEngine::Generic
    }
}

/// Finds all .exe candidate files inside directory within 3 levels of depth
pub fn find_exe_candidates(folder: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    for entry in WalkDir::new(folder)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("exe") {
                    let file_name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_lowercase())
                        .unwrap_or_default();

                    let is_ignored = IGNORED_EXE_PREFIXES.iter().any(|prefix| file_name.starts_with(prefix));
                    if !is_ignored {
                        candidates.push(path.to_path_buf());
                    }
                }
            }
        }
    }
    if candidates.is_empty() {
        // Fallback: look for setup / installer executables if no main binary was found
        for entry in WalkDir::new(folder).max_depth(3).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("exe") {
                        let file_name = path.file_name().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
                        if !file_name.starts_with("unins") && !file_name.contains("crash") {
                            candidates.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    // Sort candidates: root-level files first, then by depth, then alphabetically
    candidates.sort_by_key(|p| {
        let depth = p.strip_prefix(folder).map(|rel| rel.components().count()).unwrap_or(99);
        depth
    });

    candidates
}

/// Scores and picks the most likely main game executable
fn pick_primary_executable(folder: &Path, candidates: &[PathBuf]) -> PathBuf {
    if candidates.len() == 1 {
        return candidates[0].clone();
    }

    let folder_name = folder
        .file_name()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let mut best_score = -1i32;
    let mut best_candidate = candidates[0].clone();

    for candidate in candidates {
        let mut score = 0i32;
        let file_stem = candidate
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Exact or close match with folder name
        if !folder_name.is_empty() {
            if file_stem == folder_name {
                score += 100;
            } else if folder_name.contains(&file_stem) || file_stem.contains(&folder_name) {
                score += 50;
            }
        }

        // Favor executables directly in the root folder over nested subfolders
        if let Ok(rel) = candidate.strip_prefix(folder) {
            if rel.components().count() == 1 {
                score += 30;
            }
        }

        // Prefer larger binaries (main game binary vs tiny helper script)
        if let Ok(metadata) = candidate.metadata() {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > 5 {
                score += 20;
            } else if size_mb > 1 {
                score += 10;
            }
        }

        if score > best_score {
            best_score = score;
            best_candidate = candidate.clone();
        }
    }

    best_candidate
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_engine_heuristics_unity() {
        let temp_dir = std::env::temp_dir().join("swifter_test_unity");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("MyGame_Data")).unwrap();
        fs::write(temp_dir.join("MyGame.exe"), b"test").unwrap();

        let engine = detect_game_engine(&temp_dir);
        assert_eq!(engine, DetectedEngine::Unity);
        assert_eq!(engine.default_recommended_backend(), "DXVK");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_engine_heuristics_godot() {
        let temp_dir = std::env::temp_dir().join("swifter_test_godot");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("game.pck"), b"test").unwrap();
        fs::write(temp_dir.join("game.exe"), b"test").unwrap();

        let engine = detect_game_engine(&temp_dir);
        assert_eq!(engine, DetectedEngine::Godot);
        assert_eq!(engine.default_recommended_backend(), "DXVK");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_engine_heuristics_gamemaker() {
        let temp_dir = std::env::temp_dir().join("swifter_test_gamemaker");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("data.win"), b"test").unwrap();
        fs::write(temp_dir.join("game.exe"), b"test").unwrap();

        let engine = detect_game_engine(&temp_dir);
        assert_eq!(engine, DetectedEngine::GameMaker);
        assert_eq!(engine.default_recommended_backend(), "Vanilla");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
