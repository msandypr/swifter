<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listInstalledApps,
    launchInstalledApp,
    revealSaveFolder,
    openInFinder,
    scanAvailableExesInApp,
    updateAppTargetExe,
    type InstalledAppInfo
  } from '$lib/tauri-bridge';
  import {
    Gamepad2,
    Play,
    FolderArchive,
    FolderOpen,
    RefreshCw,
    PackagePlus,
    FileCode,
    Check,
    X
  } from 'lucide-svelte';

  let { onNavigatePacker }: { onNavigatePacker: () => void } = $props();

  let apps = $state<InstalledAppInfo[]>([]);
  let isLoading = $state(true);
  let errorMsg = $state('');

  // Target executable switcher modal states
  let isExeModalOpen = $state(false);
  let selectedAppForExe = $state<InstalledAppInfo | null>(null);
  let availableExes = $state<string[]>([]);
  let isScanningExes = $state(false);
  let chosenExe = $state('');
  let updateStatusMsg = $state('');

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

  async function handleOpenExeModal(app: InstalledAppInfo) {
    selectedAppForExe = app;
    isExeModalOpen = true;
    isScanningExes = true;
    updateStatusMsg = '';
    availableExes = [];
    chosenExe = '';

    try {
      const exes = await scanAvailableExesInApp(app.bundle_path);
      availableExes = exes;
      if (exes.length > 0) {
        chosenExe = exes[0];
      }
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isScanningExes = false;
    }
  }

  async function handleSaveTargetExe() {
    if (!selectedAppForExe || !chosenExe) return;

    try {
      await updateAppTargetExe(selectedAppForExe.bundle_path, chosenExe);
      updateStatusMsg = 'Target executable updated successfully!';
      setTimeout(() => {
        isExeModalOpen = false;
        loadApps();
      }, 700);
    } catch (err: unknown) {
      updateStatusMsg = err instanceof Error ? err.message : String(err);
    }
  }

  onMount(() => {
    loadApps();
  });
</script>

<div class="w-full max-w-4xl mx-auto flex flex-col space-y-6 pb-8">
  <!-- Top Header -->
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
        <FolderArchive class="w-4 h-4 text-zinc-400" />
        Installed Games & Apps
      </h2>
      <p class="text-xs text-zinc-400 mt-0.5">
        Games packaged with Swifter's isolated micro-prefixes
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
          You haven't packaged any games yet. Drag and drop a Windows .exe or game folder to get started.
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
    <!-- Games Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each apps as app}
        <div class="glass-card rounded-2xl p-5 flex flex-col justify-between space-y-4 border border-zinc-800 bg-zinc-900/40 transition-all">
          <div class="flex items-start gap-4">
            <div class="w-12 h-12 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0 shadow">
              <Gamepad2 class="w-6 h-6 text-zinc-300" />
            </div>

            <div class="flex-1 min-w-0 space-y-1">
              <div class="flex items-center justify-between gap-2">
                <h4 class="text-sm font-semibold text-zinc-100 truncate" title={app.name}>
                  {app.name}
                </h4>
                <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-zinc-800 text-zinc-300 border border-zinc-700 shrink-0">
                  {app.backend}
                </span>
              </div>
              <p class="text-[11px] font-mono text-zinc-500 truncate" title={app.bundle_path}>
                {app.bundle_path}
              </p>
            </div>
          </div>

          <!-- Clear Executable Target Display & Switcher -->
          <div class="flex items-center justify-between gap-2 p-2.5 rounded-xl bg-zinc-950/80 border border-zinc-800 text-[11px]">
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <FileCode class="w-3.5 h-3.5 text-zinc-400 shrink-0" />
              <span class="text-zinc-500 shrink-0 font-medium">Executable:</span>
              <span class="text-zinc-200 font-mono truncate" title={app.target_exe}>{app.target_exe || 'game.exe'}</span>
            </div>
            <button
              onclick={() => handleOpenExeModal(app)}
              class="px-2.5 py-1 rounded-lg bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-zinc-300 text-[10px] font-medium border border-zinc-700 shrink-0 transition-all"
            >
              Change .exe
            </button>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2 pt-1 border-t border-zinc-800">
            <button
              onclick={() => launchInstalledApp(app.bundle_path)}
              class="flex-1 inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 shadow-sm transition-all"
            >
              <Play class="w-3.5 h-3.5" />
              Launch
            </button>

            <button
              onclick={() => revealSaveFolder(app.slug)}
              title="Open AppData save folder"
              class="inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-xs font-medium text-zinc-300 border border-zinc-700 transition-all"
            >
              <FolderArchive class="w-3.5 h-3.5 text-zinc-400" />
              Save Data
            </button>

            <button
              onclick={() => openInFinder(app.bundle_path)}
              title="Reveal in Finder"
              class="inline-flex items-center justify-center p-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-zinc-300 border border-zinc-700 transition-all"
            >
              <FolderOpen class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- Change Target Executable Modal -->
  {#if isExeModalOpen && selectedAppForExe}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md">
      <div class="w-full max-w-md glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-9 h-9 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-300 shrink-0">
              <FileCode class="w-4 h-4" />
            </div>
            <div>
              <h3 class="text-sm font-semibold text-zinc-100">Select Launch Executable</h3>
              <p class="text-xs text-zinc-400">{selectedAppForExe.name}</p>
            </div>
          </div>

          <button onclick={() => (isExeModalOpen = false)} class="p-1 text-zinc-400 hover:text-zinc-200 rounded hover:bg-zinc-800">
            <X class="w-4 h-4" />
          </button>
        </div>

        <p class="text-xs text-zinc-400 leading-relaxed">
          If your game ran an installer or contains multiple binaries, select which executable should be launched:
        </p>

        {#if isScanningExes}
          <div class="flex items-center justify-center p-6 space-y-2 text-zinc-400">
            <div class="w-5 h-5 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
            <span class="text-xs ml-2">Scanning payload and prefix...</span>
          </div>
        {:else if availableExes.length > 0}
          <div class="space-y-2 max-h-48 overflow-y-auto">
            {#each availableExes as exe}
              <label class="flex items-center justify-between p-3 rounded-xl border cursor-pointer transition-all {chosenExe === exe ? 'border-zinc-300 bg-zinc-800 text-zinc-100' : 'border-zinc-800 bg-zinc-950/60 text-zinc-400 hover:border-zinc-700'}">
                <span class="font-mono text-xs truncate max-w-[280px]" title={exe}>{exe}</span>
                <input type="radio" name="targetExe" value={exe} bind:group={chosenExe} class="sr-only" />
                {#if chosenExe === exe}
                  <Check class="w-4 h-4 text-zinc-100 shrink-0" />
                {/if}
              </label>
            {/each}
          </div>
        {:else}
          <div class="p-3 rounded-xl bg-zinc-950 border border-zinc-800 text-xs text-zinc-400">
            No additional executables found. You can enter a relative path manually:
            <input
              type="text"
              bind:value={chosenExe}
              placeholder="e.g. drive_c/Program Files/.../game.exe"
              class="w-full mt-2 px-3 py-1.5 rounded-lg glass-input text-xs font-mono"
            />
          </div>
        {/if}

        {#if updateStatusMsg}
          <div class="text-xs font-mono text-zinc-300 px-1">{updateStatusMsg}</div>
        {/if}

        <div class="flex items-center gap-2 pt-2">
          <button
            onclick={handleSaveTargetExe}
            disabled={!chosenExe}
            class="flex-1 py-2.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-semibold active:scale-95 disabled:opacity-50 transition-all shadow-sm"
          >
            Save Launch Target
          </button>
          <button
            onclick={() => (isExeModalOpen = false)}
            class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-all"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
