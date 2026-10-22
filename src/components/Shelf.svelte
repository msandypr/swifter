<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listInstalledApps,
    launchInstalledApp,
    revealSaveFolder,
    openInFinder,
    type InstalledAppInfo
  } from '$lib/tauri-bridge';
  import {
    Gamepad2,
    Play,
    FolderArchive,
    FolderOpen,
    RefreshCw,
    PackagePlus
  } from 'lucide-svelte';

  let { onNavigatePacker }: { onNavigatePacker: () => void } = $props();

  let apps = $state<InstalledAppInfo[]>([]);
  let isLoading = $state(true);
  let errorMsg = $state('');

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

  onMount(() => {
    loadApps();
  });
</script>

<div class="w-full max-w-4xl mx-auto flex flex-col p-6 space-y-6 overflow-y-auto max-h-[calc(100vh-4rem)]">
  <!-- Top Header -->
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-lg font-semibold text-slate-100 flex items-center gap-2">
        <FolderArchive class="w-5 h-5 text-sky-400" />
        Installed Games & Apps
      </h2>
      <p class="text-xs text-slate-400 mt-0.5">
        Games packaged with Swifter's isolated micro-prefixes
      </p>
    </div>

    <button
      onclick={loadApps}
      disabled={isLoading}
      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-white/10 bg-white/5 hover:bg-white/10 text-xs font-medium text-slate-300 transition-all active:scale-95 disabled:opacity-50"
    >
      <RefreshCw class="w-3.5 h-3.5 {isLoading ? 'animate-spin' : ''}" />
      Refresh
    </button>
  </div>

  {#if errorMsg}
    <div class="p-4 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 text-xs">
      {errorMsg}
    </div>
  {/if}

  {#if isLoading && apps.length === 0}
    <div class="flex flex-col items-center justify-center p-16 space-y-3 text-slate-400">
      <div class="w-8 h-8 border-3 border-sky-400 border-t-transparent rounded-full animate-spin"></div>
      <span class="text-xs">Scanning Applications folder...</span>
    </div>
  {:else if apps.length === 0}
    <!-- Empty State -->
    <div class="glass-card rounded-2xl p-12 flex flex-col items-center justify-center text-center space-y-4">
      <div class="w-16 h-16 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center">
        <Gamepad2 class="w-8 h-8 text-slate-500" />
      </div>
      <div class="space-y-1">
        <h3 class="text-base font-semibold text-slate-200">No Swifter Games Found</h3>
        <p class="text-xs text-slate-400 max-w-sm">
          You haven't packaged any games yet. Drag-and-drop a Windows .exe or indie game folder to get started!
        </p>
      </div>
      <button
        onclick={onNavigatePacker}
        class="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-sky-600 hover:bg-sky-500 active:scale-95 text-xs font-semibold text-white shadow-lg shadow-sky-600/30 transition-all"
      >
        <PackagePlus class="w-4 h-4" />
        Package a Game
      </button>
    </div>
  {:else}
    <!-- Games Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each apps as app}
        <div class="glass-card rounded-2xl p-5 flex flex-col justify-between space-y-4 transition-all">
          <div class="flex items-start gap-4">
            <div class="w-14 h-14 rounded-xl bg-slate-900 border border-white/10 flex items-center justify-center shrink-0 shadow">
              <Gamepad2 class="w-7 h-7 text-sky-400" />
            </div>

            <div class="flex-1 min-w-0 space-y-1">
              <div class="flex items-center justify-between gap-2">
                <h4 class="text-sm font-semibold text-slate-100 truncate" title={app.name}>
                  {app.name}
                </h4>
                <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-white/10 text-slate-300 border border-white/10 shrink-0">
                  {app.backend}
                </span>
              </div>
              <p class="text-[11px] font-mono text-slate-400 truncate" title={app.bundle_path}>
                {app.bundle_path}
              </p>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2 pt-2 border-t border-white/5">
            <button
              onclick={() => launchInstalledApp(app.bundle_path)}
              class="flex-1 inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-500 active:scale-95 text-xs font-semibold text-white shadow transition-all"
            >
              <Play class="w-3.5 h-3.5" />
              Launch
            </button>

            <button
              onclick={() => revealSaveFolder(app.slug)}
              title="Open AppData save folder"
              class="inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 active:scale-95 text-xs font-medium text-slate-300 border border-white/10 transition-all"
            >
              <FolderArchive class="w-3.5 h-3.5 text-amber-400" />
              Save Data
            </button>

            <button
              onclick={() => openInFinder(app.bundle_path)}
              title="Reveal in Finder"
              class="inline-flex items-center justify-center p-1.5 rounded-xl bg-white/5 hover:bg-white/10 active:scale-95 text-slate-300 border border-white/10 transition-all"
            >
              <FolderOpen class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
