<script lang="ts">
  import {
    removeAllRuntimes,
    openSharedDriveC,
    openPrefixesDir,
    type RuntimeStatus
  } from '$lib/tauri-bridge';
  import {
    Settings,
    CheckCircle2,
    XCircle,
    Download,
    Trash2,
    Layers,
    ShieldCheck,
    RefreshCw,
    HardDrive,
    Database,
    FolderArchive
  } from 'lucide-svelte';

  let {
    runtimeStatus,
    onInstallRequest,
    onInstallEnginesRequest,
    onRemoveRequest,
    onRefresh
  }: {
    runtimeStatus: RuntimeStatus | null;
    onInstallRequest: () => void;
    onInstallEnginesRequest?: () => void;
    onRemoveRequest: () => void;
    onRefresh: () => void;
  } = $props();

  let isRefreshing = $state(false);
  let isConfirmRemoveAllOpen = $state(false);
  let isRemovingAll = $state(false);
  let removeAllMsg = $state('');

  async function handleRefreshClick() {
    isRefreshing = true;
    await onRefresh();
    setTimeout(() => {
      isRefreshing = false;
    }, 400);
  }

  async function handleConfirmRemoveAll() {
    isRemovingAll = true;
    removeAllMsg = '';
    try {
      await removeAllRuntimes();
      isConfirmRemoveAllOpen = false;
      removeAllMsg = 'All runtimes and shared caches removed.';
      await onRefresh();
    } catch (err: unknown) {
      removeAllMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isRemovingAll = false;
    }
  }
</script>

<div class="w-full max-w-2xl mx-auto flex flex-col space-y-6 pb-8 text-zinc-300">
  <!-- Header with Refresh Button -->
  <div class="flex items-center justify-between border-b border-zinc-800 pb-4">
    <div>
      <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
        <Settings class="w-4 h-4 text-zinc-400" />
        Settings
      </h2>
      <p class="text-xs text-zinc-400 mt-0.5">
        Manage Wine runtime and graphics engines.
      </p>
    </div>

    <button
      onclick={handleRefreshClick}
      disabled={isRefreshing}
      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-zinc-800 bg-zinc-900/60 hover:bg-zinc-800 text-xs font-medium text-zinc-300 transition-all active:scale-95 disabled:opacity-50 cursor-pointer"
    >
      <RefreshCw class="w-3.5 h-3.5 {isRefreshing ? 'animate-spin' : ''}" />
      Refresh
    </button>
  </div>

  {#if removeAllMsg}
    <div class="p-3 rounded-xl bg-zinc-900 border border-zinc-800 text-xs text-zinc-300">
      {removeAllMsg}
    </div>
  {/if}

  <div class="space-y-4">
    <!-- Install Card (Shown on Top When Not Installed) -->
    {#if !runtimeStatus?.is_installed}
      <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/40 space-y-3">
        <div class="space-y-1">
          <h3 class="text-xs font-semibold text-zinc-100">Install Wine Runtime</h3>
          <p class="text-xs text-zinc-400">
            Wine is required to run Windows executables and software on macOS.
          </p>
        </div>
        <div class="flex flex-wrap items-center gap-2 pt-1">
          <button
            onclick={onInstallRequest}
            class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 transition-all shadow-sm cursor-pointer"
          >
            <Download class="w-3.5 h-3.5" />
            Install Runtime (~182 MB)
          </button>
          <span class="text-xs text-zinc-500">or</span>
          <code class="px-2 py-1 rounded-lg bg-zinc-800 border border-zinc-700/60 text-[11px] text-zinc-300 font-mono select-all">
            brew install --cask wine-stable
          </code>
        </div>
      </div>
    {/if}

    <!-- Unified Runtime & Graphics Engines Card -->
    <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <!-- Wine Runtime Section -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <h3 class="text-xs font-semibold text-zinc-100">Wine Runtime</h3>
            {#if runtimeStatus?.is_installed && runtimeStatus?.runtime_name}
              <span class="text-zinc-500 text-[11px] font-mono">({runtimeStatus.runtime_name})</span>
            {/if}
          </div>
          {#if runtimeStatus?.is_installed}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              <CheckCircle2 class="w-3 h-3" />
              Installed
            </span>
          {:else}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/20">
              <XCircle class="w-3 h-3" />
              Not Installed
            </span>
          {/if}
        </div>

        {#if runtimeStatus?.is_installed}
          <div class="flex items-center justify-between gap-3 text-xs pt-1 border-t border-zinc-800/60">
            <div class="truncate text-[11px] text-zinc-400 font-mono" title={runtimeStatus.runtime_path || ''}>
              {runtimeStatus.runtime_path || '-'}
            </div>
            {#if runtimeStatus.runtime_path?.includes('/Swifter/runtimes/wine')}
              <button
                onclick={onRemoveRequest}
                class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-medium text-rose-400 hover:text-rose-300 hover:bg-rose-500/10 border border-rose-500/20 transition-all active:scale-95 cursor-pointer shrink-0"
              >
                <Trash2 class="w-3 h-3" />
                Remove
              </button>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Graphics Engines Section -->
      <div class="border-t border-zinc-800/60 pt-3 space-y-2">
        <div class="flex items-center justify-between">
          <div class="text-xs font-semibold text-zinc-100">
            Graphics Engines
          </div>
          {#if runtimeStatus?.is_installed && (!runtimeStatus?.has_dxvk || !runtimeStatus?.has_winemetalgl)}
            <button
              onclick={onInstallEnginesRequest || onInstallRequest}
              class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-[11px] font-medium border border-zinc-700 transition-all cursor-pointer active:scale-95"
            >
              <Download class="w-3 h-3 text-zinc-400" />
              <span>Install Engines</span>
            </button>
          {/if}
        </div>
        <div class="divide-y divide-zinc-800/60 text-xs">
          <!-- DXVK -->
          <div class="flex items-center justify-between py-2 first:pt-0">
            <div class="flex items-center gap-2">
              <Layers class="w-3.5 h-3.5 text-zinc-400" />
              <span class="text-zinc-200">DXVK <span class="text-zinc-500 text-[11px] font-mono">(DirectX 9–11)</span></span>
            </div>
            {#if runtimeStatus?.has_dxvk}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                <CheckCircle2 class="w-3 h-3" />
                Ready
              </span>
            {:else}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20">
                Not Installed
              </span>
            {/if}
          </div>

          <!-- Apple D3DMetal -->
          <div class="flex items-center justify-between py-2">
            <div class="flex items-center gap-2">
              <ShieldCheck class="w-3.5 h-3.5 text-zinc-400" />
              <span class="text-zinc-200">Apple D3DMetal <span class="text-zinc-500 text-[11px] font-mono">(DirectX 11/12)</span></span>
            </div>
            {#if runtimeStatus?.has_d3dmetal}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                <CheckCircle2 class="w-3 h-3" />
                Ready
              </span>
            {:else}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20">
                Not Installed
              </span>
            {/if}
          </div>

          <!-- WineMetalGL -->
          <div class="flex items-center justify-between py-2 last:pb-0">
            <div class="flex items-center gap-2">
              <Database class="w-3.5 h-3.5 text-zinc-400" />
              <span class="text-zinc-200">WineMetalGL <span class="text-zinc-500 text-[11px] font-mono">(OpenGL)</span></span>
            </div>
            {#if runtimeStatus?.has_winemetalgl}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                <CheckCircle2 class="w-3 h-3" />
                Ready
              </span>
            {:else}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20">
                Not Installed
              </span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- 3. Storage & File Access -->
    <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="text-xs font-semibold text-zinc-100">
        Storage & File Access
      </div>

      <div class="divide-y divide-zinc-800/60 text-xs">
        <div class="flex items-center justify-between py-2.5 first:pt-0 gap-3">
          <div class="min-w-0 space-y-0.5">
            <span class="text-zinc-200 font-medium block">Virtual Drive C:</span>
            <p class="text-[11px] text-zinc-500 truncate">Program Files, system32, and shared Windows runtime files</p>
          </div>
          <button
            onclick={openSharedDriveC}
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium border border-zinc-700 transition-colors cursor-pointer whitespace-nowrap shrink-0 active:scale-95"
          >
            <HardDrive class="w-3.5 h-3.5 text-zinc-400" />
            <span>Open Drive C:</span>
          </button>
        </div>

        <div class="flex items-center justify-between py-2.5 last:pb-0 gap-3">
          <div class="min-w-0 space-y-0.5">
            <span class="text-zinc-200 font-medium block">AppData & Prefixes</span>
            <p class="text-[11px] text-zinc-500 truncate">Isolated application data, user saves, and registry hives</p>
          </div>
          <button
            onclick={openPrefixesDir}
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium border border-zinc-700 transition-colors cursor-pointer whitespace-nowrap shrink-0 active:scale-95"
          >
            <FolderArchive class="w-3.5 h-3.5 text-zinc-400" />
            <span>Open AppData</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 3. Cleanup Storage -->
    <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/40 flex items-center justify-between gap-4">
      <div class="flex items-center gap-2.5">
        <HardDrive class="w-4 h-4 text-zinc-400 shrink-0" />
        <div>
          <div class="text-xs font-semibold text-zinc-100">Clean Storage</div>
          <p class="text-[11px] text-zinc-400">Remove downloaded runtimes to free disk space.</p>
        </div>
      </div>

      <button
        onclick={() => (isConfirmRemoveAllOpen = true)}
        class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-medium text-rose-400 hover:text-rose-300 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 transition-all cursor-pointer active:scale-95 shrink-0"
      >
        <Trash2 class="w-3.5 h-3.5" />
        <span>Remove All</span>
      </button>
    </div>
  </div>

  <!-- Confirm Remove All Runtimes Modal -->
  {#if isConfirmRemoveAllOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md">
      <div class="w-full max-w-sm glass-panel rounded-2xl p-5 shadow-2xl border border-zinc-800 space-y-4 animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-start gap-3">
          <div class="w-9 h-9 rounded-xl bg-zinc-800 border border-rose-500/30 flex items-center justify-center text-rose-400 shrink-0">
            <Trash2 class="w-4 h-4" />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-zinc-100">Remove All Runtimes?</h3>
            <p class="text-xs text-zinc-400 mt-0.5 leading-relaxed">
              This uninstalls Wine and shared caches (~200 MB+). Packaged games will need runtimes re-installed.
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 pt-1">
          <button
            onclick={handleConfirmRemoveAll}
            disabled={isRemovingAll}
            class="flex-1 py-2 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold active:scale-95 transition-all shadow-sm cursor-pointer disabled:opacity-50"
          >
            {isRemovingAll ? 'Deleting...' : 'Yes, Delete All'}
          </button>
          <button
            onclick={() => (isConfirmRemoveAllOpen = false)}
            disabled={isRemovingAll}
            class="px-4 py-2 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-all cursor-pointer"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
