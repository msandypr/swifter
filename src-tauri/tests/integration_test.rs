use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use swifter_lib::analyzer::folder_scan::{detect_game_engine, DetectedEngine};
use swifter_lib::engine::bundle_builder::{build_application_bundle, BuildAppOptions};
use swifter_lib::engine::prefix_mgr::get_prefix_dir;

#[test]
fn test_end_to_end_game_packaging_pipeline() {
    let temp_root = std::env::temp_dir().join("swifter_e2e_test");
    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&temp_root).unwrap();

    // 1. Create simulated indie Unity game directory
    let game_folder = temp_root.join("CelesteIndie");
    let data_folder = game_folder.join("CelesteIndie_Data");
    fs::create_dir_all(&data_folder).unwrap();
    fs::write(game_folder.join("UnityPlayer.dll"), b"MOCK_UNITY_DLL").unwrap();
    fs::write(game_folder.join("CelesteIndie.exe"), b"MOCK_EXE_PAYLOAD").unwrap();

    // 2. Test engine detection
    let engine = detect_game_engine(&game_folder);
    assert_eq!(engine, DetectedEngine::Unity);
    assert_eq!(engine.default_recommended_backend(), "DXVK");

    // 3. Test bundle assembly
    let out_dir = temp_root.join("Applications");
    fs::create_dir_all(&out_dir).unwrap();

    let build_options = BuildAppOptions {
        game_folder_path: game_folder.clone(),
        target_exe_relative: "CelesteIndie.exe".to_string(),
        app_name: "Celeste Indie".to_string(),
        backend_choice: "DXVK".to_string(),
        enable_msync: true,
        enable_esync: true,
        enable_retina: true,
        copy_files_instead_of_symlink: true,
        output_dir: Some(out_dir.clone()),
        cached_icns_path: None,
    };

    let bundle_path = build_application_bundle(build_options).expect("Packaging should succeed");
    assert!(bundle_path.exists());
    assert_eq!(bundle_path.file_name().unwrap(), "Celeste Indie.app");

    // 4. Verify bundle structure
    let contents = bundle_path.join("Contents");
    assert!(contents.join("Info.plist").exists(), "Info.plist missing");
    assert!(contents.join("PkgInfo").exists(), "PkgInfo missing");

    let resources = contents.join("Resources");
    assert!(resources.join("swifter.config.json").exists(), "Config json missing");
    assert!(resources.join("AppIcon.icns").exists(), "AppIcon.icns missing");

    let payload_exe = resources.join("game_payload/CelesteIndie.exe");
    assert!(payload_exe.exists(), "Payload executable missing");

    // 5. Verify launcher script and execution permissions
    let launcher = contents.join("MacOS/launcher");
    assert!(launcher.exists(), "Launcher binary/script missing");
    let mode = fs::metadata(&launcher).unwrap().permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "Launcher must be executable (0o755)");

    // 6. Verify launcher script syntax with bash -n
    let syntax_check = Command::new("bash")
        .arg("-n")
        .arg(&launcher)
        .output()
        .expect("Failed to run bash syntax check");
    assert!(
        syntax_check.status.success(),
        "Launcher syntax error: {}",
        String::from_utf8_lossy(&syntax_check.stderr)
    );

    // 7. Verify isolated micro-prefix creation
    let prefix = get_prefix_dir("celeste-indie");
    assert!(prefix.exists(), "Micro-prefix folder should be created");
    assert!(prefix.join("drive_c/windows").exists(), "Windows dir in prefix missing");
    assert!(prefix.join("dosdevices/c:").exists(), "dosdevices/c: symlink missing");
    assert!(prefix.join("user.reg").exists(), "user.reg missing");

    // Cleanup test artifacts
    let _ = fs::remove_dir_all(&temp_root);
}
