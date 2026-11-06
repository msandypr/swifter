import { invoke } from '@tauri-apps/api/core';

export interface SwifterInspection {
  target_exe_path: string;
  target_exe_name: string;
  game_folder_path: string;
  app_display_name: string;
  is_64bit: boolean;
  detected_engine: string;
  recommended_backend: string;
  is_anticheat_blocked: boolean;
  anticheat_reason?: string | null;
  icon_cached_path?: string | null;
  exe_candidates: string[];
  is_installer: boolean;
}

export interface PackOptions {
  game_folder_path: string;
  target_exe_name: string;
  app_name: string;
  backend_choice: string;
  enable_msync: boolean;
  copy_files_instead_of_symlink: boolean;
  launch_args?: string;
  custom_env?: Record<string, string>;
  enable_retina?: boolean;
  dpi?: number;
}

export interface InstalledAppInfo {
  name: string;
  slug: string;
  bundle_path: string;
  backend: string;
  target_exe: string;
  launch_args?: string;
  custom_env?: Record<string, string>;
  enable_retina?: boolean;
  dpi?: number;
  icon_path?: string | null;
  icon_data_url?: string | null;
}

export interface RuntimeStatus {
  is_installed: boolean;
  runtime_name?: string | null;
  runtime_path?: string | null;
  provider_upstream?: string | null;
  has_dxvk: boolean;
  has_d3dmetal: boolean;
  has_winemetalgl?: boolean;
  has_vcredist?: boolean;
  has_core_fonts?: boolean;
}

export interface RuntimeDownloadProgress {
  stage: 'downloading' | 'extracting' | 'configuring' | 'completed' | 'error';
  percent: number;
  downloaded_mb: number;
  total_mb: number;
  speed_mb: number;
  message: string;
}

export async function inspectTarget(droppedPath: string): Promise<SwifterInspection> {
  return await invoke<SwifterInspection>('inspect_target', { droppedPath });
}

export async function packToApp(options: PackOptions): Promise<string> {
  return await invoke<string>('pack_to_app', {
    gameFolderPath: options.game_folder_path,
    targetExeName: options.target_exe_name,
    appName: options.app_name,
    backendChoice: options.backend_choice,
    enableMsync: options.enable_msync,
    copyFilesInsteadOfSymlink: options.copy_files_instead_of_symlink,
    launchArgs: options.launch_args || null,
    customEnv: options.custom_env || null,
    enableRetina: options.enable_retina !== undefined ? options.enable_retina : null,
    dpi: options.dpi !== undefined ? options.dpi : null,
  });
}

export async function dryRunCheck(
  exePath: string,
  backend: string,
  launchArgs?: string | null,
  customEnv?: Record<string, string> | null
): Promise<string[]> {
  return await invoke<string[]>('dry_run_check', {
    exePath,
    backend,
    launchArgs: launchArgs || null,
    customEnv: customEnv || null,
  });
}

export async function revealSaveFolder(appSlug: string): Promise<void> {
  await invoke('reveal_save_folder', { appSlug });
}

export async function revealDriveC(appSlug: string): Promise<void> {
  await invoke('reveal_drive_c', { appSlug });
}

export async function openSharedDriveC(): Promise<void> {
  await invoke('open_shared_drive_c');
}

export async function openPrefixesDir(): Promise<void> {
  await invoke('open_prefixes_dir');
}

export async function listInstalledApps(): Promise<InstalledAppInfo[]> {
  return await invoke<InstalledAppInfo[]>('list_installed_apps');
}

export async function launchInstalledApp(bundlePath: string): Promise<void> {
  await invoke('launch_installed_app', { bundlePath });
}

export async function openInFinder(path: string): Promise<void> {
  await invoke('open_in_finder', { path });
}

export async function checkRuntimeStatus(): Promise<RuntimeStatus> {
  return await invoke<RuntimeStatus>('check_runtime_status');
}

export async function installSwifterRuntime(): Promise<string> {
  return await invoke<string>('install_swifter_runtime');
}

export async function installGraphicsEngines(): Promise<string> {
  return await invoke<string>('install_graphics_engines');
}

export async function removeSwifterRuntime(): Promise<void> {
  await invoke('remove_swifter_runtime');
}

export async function runInstallerInPrefix(exePath: string, appSlug: string): Promise<string[]> {
  return await invoke<string[]>('run_installer_in_prefix', { exePath, appSlug });
}

export async function scanAvailableExesInApp(bundlePath: string): Promise<string[]> {
  return await invoke<string[]>('scan_available_exes_in_app', { bundlePath });
}

export async function updateAppTargetExe(bundlePath: string, newTargetExe: string): Promise<void> {
  await invoke('update_app_target_exe', { bundlePath, newTargetExe });
}

export async function updateAppConfig(
  bundlePath: string,
  newTargetExe?: string | null,
  newBackend?: string | null,
  newLaunchArgs?: string | null,
  newCustomEnv?: Record<string, string> | null,
  newEnableRetina?: boolean | null,
  newDpi?: number | null
): Promise<void> {
  await invoke('update_app_config', {
    bundlePath,
    newTargetExe: newTargetExe || null,
    newBackend: newBackend || null,
    newLaunchArgs: newLaunchArgs !== undefined ? newLaunchArgs : null,
    newCustomEnv: newCustomEnv || null,
    newEnableRetina: newEnableRetina !== undefined ? newEnableRetina : null,
    newDpi: newDpi !== undefined ? newDpi : null,
  });
}

export async function installVcRedist(): Promise<string> {
  return await invoke<string>('install_vcredist');
}

export async function syncSystemFonts(): Promise<string> {
  return await invoke<string>('sync_system_fonts');
}

export async function uninstallPackagedApp(bundlePath: string): Promise<void> {
  await invoke('uninstall_packaged_app', { bundlePath });
}

export async function removeAllRuntimes(): Promise<void> {
  await invoke('remove_all_runtimes');
}

export async function launchWineTool(bundlePath: string, tool: 'winecfg' | 'regedit' | 'taskmgr' | 'control' | 'cmd'): Promise<void> {
  await invoke('launch_wine_tool', { bundlePath, tool });
}

export async function killPrefixProcesses(bundlePath: string): Promise<string> {
  return await invoke<string>('kill_prefix_processes', { bundlePath });
}

export async function openPrefixTerminal(bundlePath: string): Promise<void> {
  await invoke('open_prefix_terminal', { bundlePath });
}

export async function getAppLog(bundlePath: string): Promise<string> {
  return await invoke<string>('get_app_log', { bundlePath });
}

export async function openLogInEditor(bundlePath: string): Promise<void> {
  await invoke('open_log_in_editor', { bundlePath });
}

export async function clearAppLog(bundlePath: string): Promise<void> {
  await invoke('clear_app_log', { bundlePath });
}


export async function pickFileOrFolder(): Promise<string | null> {
  return await invoke<string | null>('pick_file_or_folder');
}
