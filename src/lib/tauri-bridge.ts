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
}

export interface PackOptions {
  game_folder_path: string;
  target_exe_name: string;
  app_name: string;
  backend_choice: string;
  enable_msync: boolean;
  copy_files_instead_of_symlink: boolean;
}

export interface InstalledAppInfo {
  name: string;
  slug: string;
  bundle_path: string;
  backend: string;
  icon_path?: string | null;
}

export interface RuntimeStatus {
  is_installed: boolean;
  runtime_name?: string | null;
  runtime_path?: string | null;
  has_dxvk: boolean;
  has_d3dmetal: boolean;
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
  });
}

export async function dryRunCheck(exePath: string, backend: string): Promise<string[]> {
  return await invoke<string[]>('dry_run_check', { exePath, backend });
}

export async function revealSaveFolder(appSlug: string): Promise<void> {
  await invoke('reveal_save_folder', { appSlug });
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

export async function removeSwifterRuntime(): Promise<void> {
  await invoke('remove_swifter_runtime');
}
