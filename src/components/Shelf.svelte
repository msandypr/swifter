<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listInstalledApps,
    launchInstalledApp,
    revealSaveFolder,
    openInFinder,
    scanAvailableExesInApp,
    updateAppTargetExe,
    uninstallPackagedApp,
    type InstalledAppInfo
  } from '$lib/tauri-bridge';
  import {
    Gamepad2,
    Rocket,
    Settings,
    FolderArchive,
    FolderOpen,
    RefreshCw,
    PackagePlus,
    FileCode,
    Check,
    X,
    Trash2,
    AlertCircle
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

<div class="w-full max-w-4xl mx-auto flex flex-col space-y-6 pb-8 text-zinc-200">
  <!-- Top Header -->
  <div class="flex items-center justify-between border-b border-zinc-800 pb-4">
    <div>
      <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
        <Gamepad2 class="w-4 h-4 text-zinc-400" />
        Game Library (Shelf)
      </h2>
      <p class="text-xs text-zinc-400 mt-0.5">
        Installed games packaged with Swifter's isolated micro-prefixes
      </p>
    </div>

    <button
      onclick={loadApps}
      disabled={isLoading}
      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-300 transition-all active:scale-95 disabled:opacity-50"
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
    <div class="glass-card rounded-2xl p-12 flex flex-col items-center justify-center text-center space-y-4 border border-zinc-800 bg-zinc-900/40">
      <div class="w-14 h-14 rounded-2xl bg-zinc-800 border border-zinc-700 flex items-center justify-center">
        <Gamepad2 class="w-7 h-7 text-zinc-500" />
      </div>
      <div class="space-y-1">
        <h3 class="text-sm font-semibold text-zinc-200">No Swifter Games Found</h3>
        <p class="text-xs text-zinc-400 max-w-sm leading-relaxed">
          You haven't packaged any games yet. Drag and drop a Windows .exe or game folder in the Packer tab to get started.
        </p>
      </div>
      <button
        onclick={onNavigatePacker}
        class="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 transition-all shadow"
      >
        <PackagePlus class="w-4 h-4" />
        Package a Game
      </button>
    </div>
  {:else}
    <!-- Games Grid: 2 columns on tablet/desktop -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each apps as app}
        <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 hover:border-zinc-700 transition-all flex flex-col justify-between space-y-4">
          <!-- Top Row: [ICON] [NAME] ... [ENGINE BADGE] -->
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-3.5 min-w-0">
              <div class="w-12 h-12 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0 shadow-sm overflow-hidden p-1">
                {#if app.icon_data_url}
                  <img src={app.icon_data_url} alt={app.name} class="w-full h-full object-contain rounded-lg" />
                {:else}
                  <Gamepad2 class="w-6 h-6 text-zinc-300" />
                {/if}
              </div>
              <div class="min-w-0">
                <h4 class="text-sm font-semibold text-zinc-100 truncate" title={app.name}>
                  {app.name}
                </h4>
                <div class="text-[11px] font-mono text-zinc-500 truncate" title={app.target_exe}>
                  {app.target_exe ? app.target_exe.split('/').pop() : 'game.exe'}
                </div>
              </div>
            </div>

            <span class="px-2.5 py-1 rounded-lg text-[10px] font-semibold bg-zinc-800 text-zinc-300 border border-zinc-700 shrink-0">
              {app.backend}
            </span>
          </div>

          <!-- Bottom Row: [LAUNCH BUTTON] [SETTINGS BUTTON] with matching rounded-xl -->
          <div class="grid grid-cols-2 gap-2.5 pt-1">
            <button
              onclick={() => launchInstalledApp(app.bundle_path)}
              class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 transition-all shadow-sm cursor-pointer"
            >
              <Rocket class="w-3.5 h-3.5 text-zinc-950" />
              <span>Launch</span>
            </button>

            <button
              onclick={() => handleOpenSettingsModal(app)}
              class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-xs font-medium text-zinc-200 border border-zinc-700 transition-all shadow-sm cursor-pointer"
            >
              <Settings class="w-3.5 h-3.5 text-zinc-400" />
              <span>Settings</span>
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
          <!-- 1. Bundle Location Row -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="text-zinc-400 font-medium">macOS Application Bundle</span>
              <button
                onclick={() => openInFinder(selectedApp?.bundle_path || '')}
                class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-[11px] border border-zinc-700 transition-colors cursor-pointer"
              >
                <FolderOpen class="w-3 h-3 text-zinc-400" />
                <span>Reveal in Finder</span>
              </button>
            </div>
            <div class="font-mono text-[11px] text-zinc-300 break-all select-all">
              {selectedApp.bundle_path}
            </div>
          </div>

          <!-- 2. Target Executable Picker -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-zinc-400 font-medium">Target Game Executable (.exe)</span>
              {#if isScanningExes}
                <div class="flex items-center gap-1.5 text-zinc-500 text-[11px]">
                  <div class="w-3 h-3 border-2 border-zinc-400 border-t-transparent rounded-full animate-spin"></div>
                  <span>Scanning...</span>
                </div>
              {/if}
            </div>

            {#if availableExes.length > 1}
              <select
                bind:value={chosenExe}
                class="w-full px-3 py-2 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 bg-zinc-900 cursor-pointer"
              >
                {#each availableExes as exe}
                  <option value={exe} class="bg-zinc-900 text-zinc-200">{exe}</option>
                {/each}
              </select>
            {:else}
              <input
                type="text"
                bind:value={chosenExe}
                class="w-full px-3 py-2 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 bg-zinc-900"
              />
            {/if}

            <p class="text-[11px] text-zinc-500">
              Ensure this points to the main game binary (inside Program Files), not the installer.
            </p>
          </div>

          <!-- 3. Save Data Row -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 flex items-center justify-between">
            <div class="space-y-0.5">
              <span class="text-zinc-400 font-medium">Save Data & Configuration</span>
              <p class="text-[11px] text-zinc-500">Access isolated AppData save files for this game.</p>
            </div>
            <button
              onclick={() => revealSaveFolder(selectedApp?.slug || '')}
              class="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs border border-zinc-700 transition-colors cursor-pointer"
            >
              <FolderArchive class="w-3.5 h-3.5 text-zinc-400" />
              <span>Open Save Data</span>
            </button>
          </div>

          <!-- 4. Uninstall Application Section -->
          <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 flex items-center justify-between">
            <div class="space-y-0.5">
              <span class="text-zinc-300 font-medium">Uninstall Game</span>
              <p class="text-[11px] text-zinc-500">Deletes the .app bundle and its micro-prefix (~15MB).</p>
            </div>
            <button
              onclick={() => (isConfirmUninstallOpen = true)}
              class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-rose-400 hover:text-rose-300 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 transition-all cursor-pointer"
            >
              <Trash2 class="w-3.5 h-3.5" />
              <span>Uninstall</span>
            </button>
          </div>
        </div>

        {#if updateStatusMsg}
          <div class="text-xs font-mono text-emerald-400 px-1">{updateStatusMsg}</div>
        {/if}

        <!-- Footer Actions with matching rounded-xl -->
        <div class="flex items-center gap-2 pt-2 border-t border-zinc-800">
          <button
            onclick={handleSaveTargetExe}
            disabled={!chosenExe}
            class="flex-1 py-2.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-semibold active:scale-95 disabled:opacity-50 transition-all shadow-sm cursor-pointer"
          >
            Save Target Executable
          </button>
          <button
            onclick={() => (isSettingsOpen = false)}
            class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-all cursor-pointer"
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
              This will permanently delete the application bundle and its isolated micro-prefix (~15MB) from your Mac.
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
