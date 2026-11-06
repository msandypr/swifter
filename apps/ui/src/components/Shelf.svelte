<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listInstalledApps,
    launchInstalledApp,
    openInFinder,
    scanAvailableExesInApp,
    updateAppTargetExe,
    updateAppConfig,
    uninstallPackagedApp,
    launchWineTool,
    killPrefixProcesses,
    openPrefixTerminal,
    getAppLog,
    openLogInEditor,
    clearAppLog,
    type InstalledAppInfo
  } from '$lib/tauri-bridge';
  import {
    Library,
    Gamepad2,
    Rocket,
    Settings,
    FolderOpen,
    RefreshCw,
    PackagePlus,
    X,
    Trash2,
    AlertCircle,
    ChevronDown,
    Layers,
    Terminal,
    Sliders,
    Database,
    Activity,
    FileText,
    SquarePower,
    Copy,
    Check,
    ExternalLink,
    Monitor
  } from 'lucide-svelte';

  let { onNavigatePacker }: { onNavigatePacker: () => void } = $props();

  let apps = $state<InstalledAppInfo[]>([]);
  let isLoading = $state(true);
  let errorMsg = $state('');

  // Game Settings Modal states
  let isSettingsOpen = $state(false);
  let selectedApp = $state<InstalledAppInfo | null>(null);
  let availableExes = $state<string[]>([]);
  let isScanningExes = $state(false);
  let chosenExe = $state('');
  let chosenBackend = $state('');
  let chosenLaunchArgs = $state('');
  let chosenEnvVars = $state('');
  let chosenRetina = $state(true);
  let chosenDpi = $state(144);
  let updateStatusMsg = $state('');

  // Quick Tools & Log states
  let isToolRunning = $state(false);
  let toolStatusMsg = $state('');
  let isLogModalOpen = $state(false);
  let logContent = $state('');
  let isLogLoading = $state(false);
  let isLogCopied = $state(false);

  // Uninstall state
  let isConfirmUninstallOpen = $state(false);
  let isUninstalling = $state(false);

  function parseEnvLines(text: string): Record<string, string> {
    const result: Record<string, string> = {};
    for (const line of text.split('\n')) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#')) continue;
      const eqIdx = trimmed.indexOf('=');
      if (eqIdx > 0) {
        const k = trimmed.slice(0, eqIdx).trim();
        const v = trimmed.slice(eqIdx + 1).trim();
        if (k) result[k] = v;
      }
    }
    return result;
  }

  function envToLines(env: Record<string, string> | undefined): string {
    if (!env) return '';
    return Object.entries(env)
      .map(([k, v]) => `${k}=${v}`)
      .join('\n');
  }

  async function loadApps() {
    isLoading = true;
    errorMsg = '';
    try {
      apps = await listInstalledApps();
    } catch (err: unknown) {
      errorMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isLoading = false;
    }
  }

  async function handleOpenSettingsModal(app: InstalledAppInfo) {
    selectedApp = app;
    isSettingsOpen = true;
    isScanningExes = true;
    updateStatusMsg = '';
    availableExes = [];
    chosenExe = app.target_exe || '';
    chosenBackend = app.backend || 'DXVK';
    chosenLaunchArgs = app.launch_args || '';
    chosenEnvVars = envToLines(app.custom_env);
    chosenRetina = app.enable_retina !== undefined ? app.enable_retina : true;
    chosenDpi = app.dpi !== undefined ? app.dpi : 144;
    try {
      const exes = await scanAvailableExesInApp(app.bundle_path);
      availableExes = exes;
      if (!chosenExe && exes.length > 0) {
        chosenExe = exes[0];
      }
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isScanningExes = false;
    }
  }

  async function handleSaveTargetExe() {
    if (!selectedApp || !chosenExe) return;

    try {
      await updateAppTargetExe(selectedApp.bundle_path, chosenExe);
      updateStatusMsg = 'Target executable saved successfully!';
      setTimeout(() => {
        isSettingsOpen = false;
        loadApps();
      }, 600);
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleSaveBackend() {
    if (!selectedApp || !chosenBackend) return;
    try {
      await updateAppConfig(selectedApp.bundle_path, null, chosenBackend);
      updateStatusMsg = 'Graphics engine updated successfully!';
      selectedApp.backend = chosenBackend;
      await loadApps();
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleSaveDisplaySettings() {
    if (!selectedApp) return;
    try {
      await updateAppConfig(selectedApp.bundle_path, null, null, null, null, chosenRetina, chosenDpi);
      updateStatusMsg = 'Display & DPI settings updated successfully!';
      selectedApp.enable_retina = chosenRetina;
      selectedApp.dpi = chosenDpi;
      await loadApps();
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleSaveLaunchArgs() {
    if (!selectedApp) return;
    try {
      await updateAppConfig(selectedApp.bundle_path, null, null, chosenLaunchArgs);
      updateStatusMsg = 'Launch arguments saved successfully!';
      selectedApp.launch_args = chosenLaunchArgs;
      await loadApps();
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleSaveEnvVars() {
    if (!selectedApp) return;
    try {
      const envObj = parseEnvLines(chosenEnvVars);
      await updateAppConfig(selectedApp.bundle_path, null, null, null, envObj);
      updateStatusMsg = 'Environment variables saved successfully!';
      selectedApp.custom_env = envObj;
      await loadApps();
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleConfirmUninstall() {
    if (!selectedApp) return;
    isUninstalling = true;
    try {
      await uninstallPackagedApp(selectedApp.bundle_path);
      isConfirmUninstallOpen = false;
      isSettingsOpen = false;
      await loadApps();
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isUninstalling = false;
    }
  }

  async function handleLaunchWineTool(tool: 'winecfg' | 'regedit' | 'taskmgr' | 'control' | 'cmd') {
    if (!selectedApp) return;
    isToolRunning = true;
    toolStatusMsg = `Launching ${tool}...`;
    try {
      await launchWineTool(selectedApp.bundle_path, tool);
      toolStatusMsg = `${tool} launched!`;
      setTimeout(() => {
        if (toolStatusMsg.includes(tool)) toolStatusMsg = '';
      }, 3500);
    } catch (err: unknown) {
      toolStatusMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isToolRunning = false;
    }
  }

  async function handleOpenTerminal() {
    if (!selectedApp) return;
    isToolRunning = true;
    toolStatusMsg = 'Opening Terminal in prefix...';
    try {
      await openPrefixTerminal(selectedApp.bundle_path);
      toolStatusMsg = 'Terminal opened!';
      setTimeout(() => {
        if (toolStatusMsg.includes('Terminal')) toolStatusMsg = '';
      }, 3500);
    } catch (err: unknown) {
      toolStatusMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isToolRunning = false;
    }
  }

  async function handleKillProcesses() {
    if (!selectedApp) return;
    isToolRunning = true;
    toolStatusMsg = 'Stopping prefix processes...';
    try {
      const res = await killPrefixProcesses(selectedApp.bundle_path);
      toolStatusMsg = res || 'Processes stopped.';
      setTimeout(() => {
        if (toolStatusMsg.includes('stopped') || toolStatusMsg.includes('terminated')) toolStatusMsg = '';
      }, 3500);
    } catch (err: unknown) {
      toolStatusMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isToolRunning = false;
    }
  }

  async function handleOpenLogModal() {
    if (!selectedApp) return;
    isLogModalOpen = true;
    isLogLoading = true;
    isLogCopied = false;
    try {
      logContent = await getAppLog(selectedApp.bundle_path);
    } catch (err: unknown) {
      logContent = `Failed to load log: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      isLogLoading = false;
    }
  }

  async function handleCopyLog() {
    if (!logContent) return;
    try {
      await navigator.clipboard.writeText(logContent);
      isLogCopied = true;
      setTimeout(() => {
        isLogCopied = false;
      }, 2000);
    } catch {
      // fallback
    }
  }

  async function handleOpenLogInEditor() {
    if (!selectedApp) return;
    try {
      await openLogInEditor(selectedApp.bundle_path);
    } catch (err: unknown) {
      logContent += `\nError opening in editor: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  async function handleClearLog() {
    if (!selectedApp) return;
    try {
      await clearAppLog(selectedApp.bundle_path);
      logContent = '=== Diagnostic log cleared ===';
    } catch (err: unknown) {
      logContent += `\nError clearing log: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  onMount(() => {
    loadApps();
  });
</script>

<div class="w-full max-w-2xl mx-auto flex flex-col space-y-6 pb-8 text-zinc-200">
  <!-- Top Header -->
  <div class="flex items-center justify-between border-b border-zinc-800 pb-4">
    <div>
      <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
        <Library class="w-4 h-4 text-zinc-400" />
        Library
      </h2>
      <p class="text-xs text-zinc-400 mt-0.5">
        Windows applications and software packaged with Swifter
      </p>
    </div>

    <button
      onclick={loadApps}
      disabled={isLoading}
      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-300 transition-all active:scale-95 disabled:opacity-50 cursor-pointer"
    >
      <RefreshCw class="w-3.5 h-3.5 {isLoading ? 'animate-spin' : ''}" />
      Refresh
    </button>
  </div>

  {#if errorMsg}
    <div class="p-4 rounded-xl bg-zinc-900 border border-rose-500/30 text-rose-300 text-xs">
      {errorMsg}
    </div>
  {/if}

  {#if isLoading && apps.length === 0}
    <div class="flex flex-col items-center justify-center p-16 space-y-3 text-zinc-400">
      <div class="w-6 h-6 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
      <span class="text-xs">Scanning Applications folder...</span>
    </div>
  {:else if apps.length === 0}
    <!-- Empty State -->
    <div class="glass-card rounded-2xl p-10 flex flex-col items-center justify-center text-center space-y-4 border border-zinc-800 bg-zinc-900/40">
      <div class="w-12 h-12 rounded-2xl bg-zinc-800 border border-zinc-700 flex items-center justify-center">
        <Library class="w-6 h-6 text-zinc-500" />
      </div>
      <div class="space-y-1">
        <h3 class="text-sm font-semibold text-zinc-200">No Applications Found</h3>
        <p class="text-xs text-zinc-400 max-w-sm leading-relaxed">
          You haven't converted any applications yet. Drop a Windows .exe or folder in the Convert tab to get started.
        </p>
      </div>
      <button
        onclick={onNavigatePacker}
        class="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 transition-all shadow cursor-pointer"
      >
        <PackagePlus class="w-4 h-4" />
        Convert an App
      </button>
    </div>
  {:else}
    <!-- Apps List: 1 column centered -->
    <div class="flex flex-col space-y-3 w-full">
      {#each apps as app}
        <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/40 hover:border-zinc-700 transition-all flex flex-col space-y-3">
          <!-- Top Row: [ICON] [NAME] ... [ENGINE BADGE] -->
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-11 h-11 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0 shadow-sm overflow-hidden p-1">
                {#if app.icon_data_url}
                  <img src={app.icon_data_url} alt={app.name} class="w-full h-full object-contain rounded-lg" />
                {:else}
                  <Gamepad2 class="w-5 h-5 text-zinc-300" />
                {/if}
              </div>
              <div class="min-w-0">
                <h4 class="text-sm font-semibold text-zinc-100 truncate" title={app.name}>
                  {app.name}
                </h4>
                <div class="text-[11px] font-mono text-zinc-500 truncate" title={app.target_exe}>
                  {app.target_exe ? app.target_exe.split('/').pop() : 'app.exe'}
                </div>
              </div>
            </div>

            <span class="px-2 py-0.5 rounded-lg text-[10px] font-semibold bg-zinc-800 text-zinc-300 border border-zinc-700 shrink-0">
              {app.backend}
            </span>
          </div>

          <!-- Bottom Row: [LAUNCH BUTTON] [SETTINGS BUTTON] -->
          <div class="grid grid-cols-2 gap-2 pt-0.5">
            <button
              onclick={() => launchInstalledApp(app.bundle_path)}
              class="inline-flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 transition-all shadow-sm cursor-pointer"
            >
              <Rocket class="w-3.5 h-3.5 text-zinc-950" />
              <span>Launch</span>
            </button>

            <button
              onclick={() => handleOpenSettingsModal(app)}
              class="inline-flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-xs font-medium text-zinc-200 border border-zinc-700 transition-all shadow-sm cursor-pointer"
            >
              <Settings class="w-3.5 h-3.5 text-zinc-400" />
              <span>Options</span>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- Game Settings Modal -->
  {#if isSettingsOpen && selectedApp}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md"
      onclick={(e) => { if (e.target === e.currentTarget) isSettingsOpen = false; }}
      role="presentation"
    >
      <div
        class="w-full max-w-lg max-h-[85vh] glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 flex flex-col animate-in fade-in zoom-in-95 duration-150"
        role="dialog"
        aria-labelledby="options-modal-title"
        aria-modal="true"
      >
        <!-- Modal Header -->
        <div class="flex items-center justify-between pb-3.5 border-b border-zinc-800 shrink-0">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-300 shrink-0 overflow-hidden p-1">
              {#if selectedApp.icon_data_url}
                <img src={selectedApp.icon_data_url} alt={selectedApp.name} class="w-full h-full object-contain rounded-lg" />
              {:else}
                <Settings class="w-5 h-5" />
              {/if}
            </div>
            <div>
              <h3 id="options-modal-title" class="text-sm font-semibold text-zinc-100">{selectedApp.name}</h3>
              <p class="text-xs text-zinc-400">Application Configuration & Paths</p>
            </div>
          </div>

          <button
            onclick={() => (isSettingsOpen = false)}
            aria-label="Close Options"
            class="p-1.5 text-zinc-400 hover:text-zinc-200 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Scrollable Options Body -->
        <div class="flex-1 min-h-0 overflow-y-auto py-3.5 pr-1.5 space-y-3 text-xs overscroll-contain">
          <!-- 1. Bundle Location & Reveal in Finder -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 flex items-center justify-between gap-3">
            <div class="min-w-0 space-y-0.5">
              <span class="text-zinc-300 font-medium block">Application Bundle</span>
              <div class="font-mono text-[11px] text-zinc-500 truncate" title={selectedApp.bundle_path}>
                {selectedApp.bundle_path}
              </div>
            </div>
            <button
              onclick={() => openInFinder(selectedApp?.bundle_path || '')}
              class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium border border-zinc-700 transition-colors cursor-pointer whitespace-nowrap shrink-0 active:scale-95"
            >
              <FolderOpen class="w-3.5 h-3.5 text-zinc-400" />
              <span>Reveal in Finder</span>
            </button>
          </div>

          <!-- 2. Target Executable Picker -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-zinc-300 font-medium">Target Executable (.exe)</span>
              {#if isScanningExes}
                <div class="flex items-center gap-1.5 text-zinc-500 text-[11px]">
                  <div class="w-3 h-3 border-2 border-zinc-400 border-t-transparent rounded-full animate-spin"></div>
                  <span>Scanning...</span>
                </div>
              {/if}
            </div>

            {#if availableExes.length > 1}
              <div class="relative">
                <select
                  bind:value={chosenExe}
                  class="w-full h-10 pl-3 pr-9 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 hover:border-zinc-700 focus:border-zinc-600 bg-zinc-900 cursor-pointer appearance-none transition-colors"
                >
                  {#each availableExes as exe}
                    <option value={exe} class="bg-zinc-900 text-zinc-200">{exe}</option>
                  {/each}
                </select>
                <div class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-zinc-400">
                  <ChevronDown class="w-3.5 h-3.5" />
                </div>
              </div>
            {:else}
              <input
                type="text"
                bind:value={chosenExe}
                class="w-full h-10 px-3 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 bg-zinc-900"
              />
            {/if}

            <div class="flex items-center justify-between pt-1 gap-2">
              <p class="text-[11px] text-zinc-500">
                Ensure this points to the main application binary, not the installer.
              </p>
              <button
                onclick={handleSaveTargetExe}
                disabled={!chosenExe || chosenExe === selectedApp.target_exe}
                class="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 disabled:opacity-40 transition-all shadow-sm cursor-pointer whitespace-nowrap shrink-0"
              >
                Save Executable
              </button>
            </div>
            {#if updateStatusMsg && updateStatusMsg.toLowerCase().includes('executable')}
              <div class="text-[11px] font-mono text-emerald-400 pt-0.5">{updateStatusMsg}</div>
            {/if}
          </div>

          <!-- 3. Graphics Engine Backend Selector -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2.5">
            <div class="flex items-center justify-between">
              <span class="text-zinc-300 font-medium flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-zinc-400" />
                Graphics Engine Backend
              </span>
              <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-zinc-800 text-zinc-300 border border-zinc-700">
                Current: {selectedApp.backend}
              </span>
            </div>

            <div class="grid grid-cols-3 gap-2 text-xs">
              <button
                type="button"
                onclick={() => (chosenBackend = 'DXVK')}
                class="py-2 px-2 rounded-xl border text-center font-medium transition-all cursor-pointer {chosenBackend === 'DXVK' ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/60 text-zinc-400'}"
              >
                <div>DXVK</div>
                <div class="text-[10px] text-zinc-500 font-mono">DirectX 9–11</div>
              </button>

              <button
                type="button"
                onclick={() => (chosenBackend = 'D3DMetal')}
                class="py-2 px-2 rounded-xl border text-center font-medium transition-all cursor-pointer {chosenBackend === 'D3DMetal' ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/60 text-zinc-400'}"
              >
                <div>D3DMetal</div>
                <div class="text-[10px] text-zinc-500 font-mono">DirectX 11/12</div>
              </button>

              <button
                type="button"
                onclick={() => (chosenBackend = 'Vanilla')}
                class="py-2 px-2 rounded-xl border text-center font-medium transition-all cursor-pointer {chosenBackend === 'Vanilla' ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/60 text-zinc-400'}"
              >
                <div>Vanilla</div>
                <div class="text-[10px] text-zinc-500 font-mono">OpenGL / Legacy</div>
              </button>
            </div>

            {#if chosenBackend === 'D3DMetal'}
              <div class="text-[11px] text-amber-400/90 bg-amber-950/20 p-2 rounded-lg border border-amber-500/20">
                Notice: D3DMetal requires Apple Silicon and Apple Game Porting Toolkit (GPTK) libraries (e.g. from Whisky).
              </div>
            {/if}

            <div class="flex items-center justify-between pt-1">
              <p class="text-[11px] text-zinc-500">
                Switch translation layer without reinstalling the application.
              </p>
              <button
                onclick={handleSaveBackend}
                disabled={!chosenBackend || chosenBackend === selectedApp.backend}
                class="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 disabled:opacity-40 transition-all shadow-sm cursor-pointer whitespace-nowrap shrink-0 ml-2"
              >
                Apply Engine
              </button>
            </div>
            {#if updateStatusMsg && updateStatusMsg.toLowerCase().includes('engine')}
              <div class="text-[11px] font-mono text-emerald-400 pt-0.5">{updateStatusMsg}</div>
            {/if}
          </div>

          <!-- 4. Display & High-DPI Scaling -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2.5">
            <div class="flex items-center justify-between">
              <span class="text-zinc-300 font-medium flex items-center gap-1.5">
                <Monitor class="w-3.5 h-3.5 text-zinc-400" />
                Display & High-DPI Scaling
              </span>
              <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-zinc-800 text-zinc-300 border border-zinc-700">
                {selectedApp.enable_retina ? 'Retina On' : 'Retina Off'} · {selectedApp.dpi || 144} DPI
              </span>
            </div>

            <!-- Retina Mode Toggle -->
            <label class="flex items-center justify-between p-2.5 rounded-xl bg-zinc-900/60 hover:bg-zinc-900 border border-zinc-800/80 transition-colors cursor-pointer">
              <div class="space-y-0.5 pr-3">
                <div class="text-xs font-semibold text-zinc-200">High-DPI Retina Mode</div>
                <div class="text-[11px] text-zinc-400">
                  Enables crisp native pixel density rendering via macOS window backing store.
                </div>
              </div>
              <input type="checkbox" bind:checked={chosenRetina} class="w-4 h-4 rounded accent-zinc-400 shrink-0" />
            </label>

            <!-- DPI Scaling Presets -->
            <div class="space-y-1.5 pt-1">
              <div class="flex items-center justify-between">
                <span class="text-[11px] font-medium text-zinc-400">Windows UI & Font Scaling (DPI)</span>
                <span class="text-[10px] font-mono text-zinc-500">
                  {chosenDpi === 96 ? '100% (Standard)' : chosenDpi === 120 ? '125% (Medium)' : chosenDpi === 144 ? '150% (Recommended)' : chosenDpi === 192 ? '200% (Large)' : `${chosenDpi} DPI`}
                </span>
              </div>
              <div class="grid grid-cols-4 gap-2 text-xs">
                <button
                  type="button"
                  onclick={() => (chosenDpi = 96)}
                  class="py-2 px-1 rounded-xl border text-center font-medium transition-all cursor-pointer {chosenDpi === 96 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/60 text-zinc-400'}"
                >
                  <div class="font-semibold text-xs">100%</div>
                  <div class="text-[10px] text-zinc-500 font-mono">96 DPI</div>
                </button>

                <button
                  type="button"
                  onclick={() => (chosenDpi = 120)}
                  class="py-2 px-1 rounded-xl border text-center font-medium transition-all cursor-pointer {chosenDpi === 120 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/60 text-zinc-400'}"
                >
                  <div class="font-semibold text-xs">125%</div>
                  <div class="text-[10px] text-zinc-500 font-mono">120 DPI</div>
                </button>

                <button
                  type="button"
                  onclick={() => (chosenDpi = 144)}
                  class="py-2 px-1 rounded-xl border text-center font-medium transition-all cursor-pointer {chosenDpi === 144 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/60 text-zinc-400'}"
                >
                  <div class="font-semibold text-xs">150%</div>
                  <div class="text-[10px] text-zinc-500 font-mono">144 DPI</div>
                </button>

                <button
                  type="button"
                  onclick={() => (chosenDpi = 192)}
                  class="py-2 px-1 rounded-xl border text-center font-medium transition-all cursor-pointer {chosenDpi === 192 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/60 text-zinc-400'}"
                >
                  <div class="font-semibold text-xs">200%</div>
                  <div class="text-[10px] text-zinc-500 font-mono">192 DPI</div>
                </button>
              </div>
            </div>

            <div class="flex items-center justify-between pt-1">
              <p class="text-[11px] text-zinc-500">
                Fixes blurry or microscopic in-game UI & text on Apple Retina displays.
              </p>
              <button
                onclick={handleSaveDisplaySettings}
                disabled={chosenRetina === selectedApp.enable_retina && chosenDpi === (selectedApp.dpi ?? 144)}
                class="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 disabled:opacity-40 transition-all shadow-sm cursor-pointer whitespace-nowrap shrink-0 ml-2"
              >
                Apply Display
              </button>
            </div>
            {#if updateStatusMsg && updateStatusMsg.toLowerCase().includes('dpi')}
              <div class="text-[11px] font-mono text-emerald-400 pt-0.5">{updateStatusMsg}</div>
            {/if}
          </div>

          <!-- 5. Quick Tools & Troubleshooting -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2.5">
            <div class="flex items-center justify-between">
              <span class="text-zinc-300 font-medium flex items-center gap-1.5">
                <Sliders class="w-3.5 h-3.5 text-zinc-400" />
                Quick Tools & Troubleshooting
              </span>
              {#if toolStatusMsg}
                <span class="text-[11px] font-mono text-emerald-400 truncate max-w-[200px]">{toolStatusMsg}</span>
              {/if}
            </div>
            <p class="text-[11px] text-zinc-500">
              Run Wine configuration utilities or inspect diagnostic output for this prefix.
            </p>

            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 pt-1 text-xs">
              <!-- Winecfg -->
              <button
                type="button"
                disabled={isToolRunning}
                onclick={() => handleLaunchWineTool('winecfg')}
                class="flex items-center gap-2 p-2.5 rounded-xl bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-200 transition-all cursor-pointer active:scale-95 disabled:opacity-50 text-left"
              >
                <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0">
                  <Sliders class="w-3.5 h-3.5 text-indigo-400" />
                </div>
                <div class="min-w-0">
                  <div class="font-semibold text-zinc-200 text-xs">Winecfg</div>
                  <div class="text-[10px] text-zinc-500 truncate">Config & Audio</div>
                </div>
              </button>

              <!-- Regedit -->
              <button
                type="button"
                disabled={isToolRunning}
                onclick={() => handleLaunchWineTool('regedit')}
                class="flex items-center gap-2 p-2.5 rounded-xl bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-200 transition-all cursor-pointer active:scale-95 disabled:opacity-50 text-left"
              >
                <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0">
                  <Database class="w-3.5 h-3.5 text-cyan-400" />
                </div>
                <div class="min-w-0">
                  <div class="font-semibold text-zinc-200 text-xs">Regedit</div>
                  <div class="text-[10px] text-zinc-500 truncate">Registry Keys</div>
                </div>
              </button>

              <!-- Task Manager -->
              <button
                type="button"
                disabled={isToolRunning}
                onclick={() => handleLaunchWineTool('taskmgr')}
                class="flex items-center gap-2 p-2.5 rounded-xl bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-200 transition-all cursor-pointer active:scale-95 disabled:opacity-50 text-left"
              >
                <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0">
                  <Activity class="w-3.5 h-3.5 text-amber-400" />
                </div>
                <div class="min-w-0">
                  <div class="font-semibold text-zinc-200 text-xs">Task Manager</div>
                  <div class="text-[10px] text-zinc-500 truncate">Wine Tasks</div>
                </div>
              </button>

              <!-- Terminal Shell -->
              <button
                type="button"
                disabled={isToolRunning}
                onclick={handleOpenTerminal}
                class="flex items-center gap-2 p-2.5 rounded-xl bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-200 transition-all cursor-pointer active:scale-95 disabled:opacity-50 text-left"
              >
                <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0">
                  <Terminal class="w-3.5 h-3.5 text-emerald-400" />
                </div>
                <div class="min-w-0">
                  <div class="font-semibold text-zinc-200 text-xs">Terminal</div>
                  <div class="text-[10px] text-zinc-500 truncate">Shell in Prefix</div>
                </div>
              </button>

              <!-- View Logs -->
              <button
                type="button"
                onclick={handleOpenLogModal}
                class="flex items-center gap-2 p-2.5 rounded-xl bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-200 transition-all cursor-pointer active:scale-95 text-left"
              >
                <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0">
                  <FileText class="w-3.5 h-3.5 text-blue-400" />
                </div>
                <div class="min-w-0">
                  <div class="font-semibold text-zinc-200 text-xs">View Logs</div>
                  <div class="text-[10px] text-zinc-500 truncate">last_run.log</div>
                </div>
              </button>

              <!-- Force Stop -->
              <button
                type="button"
                disabled={isToolRunning}
                onclick={handleKillProcesses}
                class="flex items-center gap-2 p-2.5 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 text-rose-300 transition-all cursor-pointer active:scale-95 disabled:opacity-50 text-left"
              >
                <div class="w-7 h-7 rounded-lg bg-rose-500/20 border border-rose-500/30 flex items-center justify-center shrink-0">
                  <SquarePower class="w-3.5 h-3.5 text-rose-400" />
                </div>
                <div class="min-w-0">
                  <div class="font-semibold text-rose-200 text-xs">Force Stop</div>
                  <div class="text-[10px] text-rose-400/80 truncate">Kill wineserver</div>
                </div>
              </button>
            </div>
          </div>

          <!-- 5. Launch Arguments -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2.5">
            <div class="flex items-center justify-between">
              <span class="text-zinc-300 font-medium flex items-center gap-1.5">
                <Terminal class="w-3.5 h-3.5 text-zinc-400" />
                Launch Arguments
              </span>
            </div>
            <input
              type="text"
              bind:value={chosenLaunchArgs}
              placeholder="e.g. -novid -windowed -force-d3d11"
              class="w-full h-10 px-3 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 bg-zinc-900"
            />
            <div class="flex items-center justify-between pt-1">
              <p class="text-[11px] text-zinc-500">
                Command-line flags passed directly to binary.
              </p>
              <button
                onclick={handleSaveLaunchArgs}
                disabled={chosenLaunchArgs === (selectedApp.launch_args || '')}
                class="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 disabled:opacity-40 transition-all shadow-sm cursor-pointer whitespace-nowrap shrink-0 ml-2"
              >
                Save Arguments
              </button>
            </div>
            {#if updateStatusMsg && updateStatusMsg.toLowerCase().includes('argument')}
              <div class="text-[11px] font-mono text-emerald-400 pt-0.5">{updateStatusMsg}</div>
            {/if}
          </div>

          <!-- 5. Custom Environment Variables -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2.5">
            <div class="flex items-center justify-between">
              <span class="text-zinc-300 font-medium flex items-center gap-1.5">
                <Terminal class="w-3.5 h-3.5 text-zinc-400" />
                Environment Variables
              </span>
            </div>
            <textarea
              bind:value={chosenEnvVars}
              rows={3}
              placeholder="DXVK_HUD=fps&#10;MVK_CONFIG_LOG_LEVEL=1"
              class="w-full p-2.5 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 bg-zinc-900 resize-y"
            ></textarea>
            <div class="flex items-center justify-between pt-1">
              <p class="text-[11px] text-zinc-500">
                Format: KEY=VALUE (one per line).
              </p>
              <button
                onclick={handleSaveEnvVars}
                disabled={chosenEnvVars === envToLines(selectedApp.custom_env)}
                class="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 disabled:opacity-40 transition-all shadow-sm cursor-pointer whitespace-nowrap shrink-0 ml-2"
              >
                Save Env
              </button>
            </div>
            {#if updateStatusMsg && updateStatusMsg.toLowerCase().includes('environment')}
              <div class="text-[11px] font-mono text-emerald-400 pt-0.5">{updateStatusMsg}</div>
            {/if}
          </div>

          <!-- 6. Uninstall Application Section -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 flex items-center justify-between gap-3">
            <div class="min-w-0 space-y-0.5">
              <span class="text-zinc-300 font-medium block">Uninstall Application</span>
              <p class="text-[11px] text-zinc-500 truncate">Deletes the .app bundle and its micro-prefix.</p>
            </div>
            <button
              onclick={() => (isConfirmUninstallOpen = true)}
              class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-rose-400 hover:text-rose-300 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 whitespace-nowrap shrink-0 transition-all cursor-pointer active:scale-95"
            >
              <Trash2 class="w-3.5 h-3.5" />
              <span>Uninstall</span>
            </button>
          </div>
        </div>

        <!-- Footer Actions -->
        <div class="shrink-0 pt-3 border-t border-zinc-800 space-y-2">
          {#if updateStatusMsg}
            <div class="text-xs font-mono text-emerald-400 px-1">{updateStatusMsg}</div>
          {/if}
          <div class="flex items-center justify-end">
            <button
              onclick={() => (isSettingsOpen = false)}
              class="w-full py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-zinc-200 text-xs font-semibold border border-zinc-700 transition-all shadow-sm cursor-pointer"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Diagnostic Log Viewer Modal -->
  {#if isLogModalOpen && selectedApp}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md"
      onclick={(e) => { if (e.target === e.currentTarget) isLogModalOpen = false; }}
      role="presentation"
    >
      <div
        class="w-full max-w-2xl max-h-[85vh] glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 flex flex-col animate-in fade-in zoom-in-95 duration-150"
        role="dialog"
        aria-labelledby="log-modal-title"
        aria-modal="true"
      >
        <!-- Modal Header -->
        <div class="flex items-center justify-between pb-3.5 border-b border-zinc-800 shrink-0">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-blue-400 shrink-0">
              <FileText class="w-5 h-5" />
            </div>
            <div>
              <h3 id="log-modal-title" class="text-sm font-semibold text-zinc-100">
                Diagnostic Logs: {selectedApp.name}
              </h3>
              <p class="text-xs text-zinc-400 font-mono">last_run.log in micro-prefix</p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <button
              onclick={handleCopyLog}
              disabled={isLogLoading || !logContent}
              class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium border border-zinc-700 transition-colors cursor-pointer active:scale-95 disabled:opacity-50"
            >
              {#if isLogCopied}
                <Check class="w-3.5 h-3.5 text-emerald-400" />
                <span class="text-emerald-400 font-medium">Copied!</span>
              {:else}
                <Copy class="w-3.5 h-3.5 text-zinc-400" />
                <span>Copy</span>
              {/if}
            </button>

            <button
              onclick={handleOpenLogInEditor}
              title="Open in TextEdit"
              class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium border border-zinc-700 transition-colors cursor-pointer active:scale-95"
            >
              <ExternalLink class="w-3.5 h-3.5 text-zinc-400" />
              <span>Editor</span>
            </button>

            <button
              onclick={() => (isLogModalOpen = false)}
              aria-label="Close Logs"
              class="p-1.5 text-zinc-400 hover:text-zinc-200 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer"
            >
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Log Content Body -->
        <div class="flex-1 min-h-0 overflow-y-auto my-4 p-4 rounded-xl bg-zinc-950/90 border border-zinc-800 text-zinc-300 font-mono text-[11px] leading-relaxed whitespace-pre-wrap select-text overscroll-contain">
          {#if isLogLoading}
            <div class="flex items-center justify-center p-8 text-zinc-500 gap-2">
              <RefreshCw class="w-4 h-4 animate-spin" />
              <span>Reading diagnostic log...</span>
            </div>
          {:else}
            {logContent}
          {/if}
        </div>

        <!-- Log Footer -->
        <div class="shrink-0 pt-3 border-t border-zinc-800 flex items-center justify-between">
          <button
            onclick={handleClearLog}
            disabled={isLogLoading || !logContent}
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-rose-400 hover:text-rose-300 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 text-xs font-medium transition-all cursor-pointer disabled:opacity-50"
          >
            <Trash2 class="w-3.5 h-3.5" />
            <span>Clear Log</span>
          </button>

          <div class="flex items-center gap-2">
            <button
              onclick={handleOpenLogModal}
              disabled={isLogLoading}
              class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium border border-zinc-700 transition-colors cursor-pointer active:scale-95"
            >
              <RefreshCw class="w-3.5 h-3.5 {isLogLoading ? 'animate-spin' : ''}" />
              <span>Refresh</span>
            </button>

            <button
              onclick={() => (isLogModalOpen = false)}
              class="px-5 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 transition-all shadow-sm cursor-pointer"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Confirm Uninstall Modal -->
  {#if isConfirmUninstallOpen && selectedApp}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md"
      onclick={(e) => { if (e.target === e.currentTarget && !isUninstalling) isConfirmUninstallOpen = false; }}
      role="presentation"
    >
      <div class="w-full max-w-sm glass-panel rounded-2xl p-5 shadow-2xl border border-zinc-800 space-y-4 animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-start gap-3">
          <div class="w-9 h-9 rounded-xl bg-zinc-800 border border-rose-500/30 flex items-center justify-center text-rose-400 shrink-0">
            <Trash2 class="w-4 h-4" />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-zinc-100">Uninstall {selectedApp.name}?</h3>
            <p class="text-xs text-zinc-400 mt-0.5 leading-relaxed">
              This will permanently delete the application bundle and its isolated micro-prefix from your Mac.
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 pt-1">
          <button
            onclick={handleConfirmUninstall}
            disabled={isUninstalling}
            class="flex-1 py-2 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold active:scale-95 transition-all shadow-sm cursor-pointer disabled:opacity-50"
          >
            {isUninstalling ? 'Uninstalling...' : 'Yes, Delete Game'}
          </button>
          <button
            onclick={() => (isConfirmUninstallOpen = false)}
            disabled={isUninstalling}
            class="px-4 py-2 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-all cursor-pointer"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
