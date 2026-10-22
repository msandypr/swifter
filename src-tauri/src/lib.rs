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
            commands::list_installed_apps,
            commands::launch_installed_app,
            commands::open_in_finder,
            commands::check_runtime_status,
            commands::install_swifter_runtime,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Swifter application");
}
