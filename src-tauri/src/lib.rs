pub mod analyzer;
pub mod commands;
pub mod engine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::inspect_target,
            commands::pack_to_app,
            commands::dry_run_check,
            commands::reveal_save_folder,
            commands::reveal_drive_c,
            commands::open_shared_drive_c,
            commands::open_prefixes_dir,
            commands::list_installed_apps,
            commands::launch_installed_app,
            commands::open_in_finder,
            commands::check_runtime_status,
            commands::install_swifter_runtime,
            commands::remove_swifter_runtime,
            commands::run_installer_in_prefix,
            commands::scan_available_exes_in_app,
            commands::update_app_target_exe,
            commands::update_app_config,
            commands::uninstall_packaged_app,
            commands::remove_all_runtimes,
            commands::pick_file_or_folder,
            commands::install_graphics_engines,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Swifter application");
}
