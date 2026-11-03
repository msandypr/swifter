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
    Layers
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
  let updateStatusMsg = $state('');
  // Uninstall state
  let isConfirmUninstallOpen = $state(false);
  let isUninstalling = $state(false);

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
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md">
      <div class="w-full max-w-lg glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-150">
        <!-- Modal Header -->
        <div class="flex items-center justify-between pb-2 border-b border-zinc-800">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-300 shrink-0 overflow-hidden p-1">
              {#if selectedApp.icon_data_url}
                <img src={selectedApp.icon_data_url} alt={selectedApp.name} class="w-full h-full object-contain rounded-lg" />
              {:else}
                <Settings class="w-5 h-5" />
              {/if}
            </div>
            <div>
              <h3 class="text-sm font-semibold text-zinc-100">{selectedApp.name}</h3>
              <p class="text-xs text-zinc-400">Application Configuration & Paths</p>
            </div>
          </div>

          <button onclick={() => (isSettingsOpen = false)} class="p-1 text-zinc-400 hover:text-zinc-200 rounded-lg hover:bg-zinc-800 transition-colors">
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="space-y-3 text-xs">
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
          <!-- 4. Uninstall Application Section -->
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

        {#if updateStatusMsg}
          <div class="text-xs font-mono text-emerald-400 px-1">{updateStatusMsg}</div>
        {/if}

        <!-- Footer Actions -->
        <div class="flex items-center justify-end pt-2 border-t border-zinc-800">
          <button
            onclick={() => (isSettingsOpen = false)}
            class="w-full py-2.5 rounded-xl bg-rose-600 hover:bg-rose-500 active:scale-95 text-white text-xs font-semibold transition-all shadow-sm cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Confirm Uninstall Modal -->
  {#if isConfirmUninstallOpen && selectedApp}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md">
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
