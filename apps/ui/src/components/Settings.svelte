<script lang="ts">
  import {
    removeAllRuntimes,
    openSharedDriveC,
    openPrefixesDir,
    type RuntimeStatus
  } from '$lib/tauri-bridge';
  import {
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
    onInstallVcRedistRequest,
    onSyncFontsRequest,
    onRemoveRequest,
    onRefresh
  }: {
    runtimeStatus: RuntimeStatus | null;
    onInstallRequest: () => void;
    onInstallEnginesRequest?: () => void;
    onInstallVcRedistRequest?: () => void;
    onSyncFontsRequest?: () => void;
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

<div class="w-full flex flex-col space-y-4 pb-6 text-zinc-300">
  <!-- Compact Action Row -->
  <div class="flex items-center justify-between">
    <p class="text-xs text-zinc-400">
      Manage Wine runtime, graphics engines, and shared storage.
    </p>

    <button
      onclick={handleRefreshClick}
      disabled={isRefreshing}
      class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg border border-white/[0.08] bg-white/[0.04] hover:bg-white/[0.08] text-xs font-medium text-zinc-300 transition-colors disabled:opacity-50 cursor-pointer"
    >
      <RefreshCw class="w-3 h-3 {isRefreshing ? 'animate-spin' : ''}" />
      Refresh
    </button>
  </div>

  {#if removeAllMsg}
    <div class="p-3 rounded-xl bg-zinc-900 border border-zinc-800 text-xs text-zinc-300">
      {removeAllMsg}
    </div>
  {/if}

  <div class="space-y-5">
    <!-- Section 1: Wine Runtime -->
    <section class="space-y-2">
      <div class="flex items-center justify-between px-1">
        <h3 class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">Wine Runtime</h3>
        {#if runtimeStatus?.is_installed}
          <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <CheckCircle2 class="w-3 h-3" />
            Installed
          </span>
        {:else}
          <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/20">
            <XCircle class="w-3 h-3" />
            Not Installed
          </span>
        {/if}
      </div>

      <div class="rounded-xl border border-white/[0.07] bg-white/[0.025] p-4 space-y-3">
        {#if runtimeStatus?.is_installed}
          <div class="flex items-center justify-between gap-3">
            <div class="min-w-0 space-y-0.5">
              <div class="text-xs font-medium text-zinc-100">
                {runtimeStatus.runtime_name || 'Wine Runtime'}
              </div>
              <div class="truncate text-[11px] text-zinc-400 font-mono" title={runtimeStatus.runtime_path || ''}>
                {runtimeStatus.runtime_path || '-'}
              </div>
            </div>
            {#if runtimeStatus.runtime_path?.includes('/Swifter/runtimes/wine')}
              <button
                onclick={onRemoveRequest}
                class="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium text-rose-400 hover:text-rose-300 hover:bg-rose-500/10 border border-rose-500/20 transition-colors cursor-pointer shrink-0"
              >
                <Trash2 class="w-3 h-3" />
                Remove
              </button>
            {/if}
          </div>
        {:else}
          <div class="space-y-2.5">
            <p class="text-xs text-zinc-400">
              Wine is required to run Windows executables and software on macOS.
            </p>
            <div class="flex flex-wrap items-center gap-2">
              <button
                onclick={onInstallRequest}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs transition-colors cursor-pointer"
              >
                <Download class="w-3.5 h-3.5" />
                Install Runtime (~182 MB)
              </button>
              <span class="text-xs text-zinc-500">or</span>
              <code class="px-2 py-1 rounded-md bg-zinc-900 border border-white/[0.08] text-[11px] text-zinc-300 font-mono select-all">
                brew install --cask wine-stable
              </code>
            </div>
          </div>
        {/if}
      </div>
    </section>

    <!-- Section 2: Graphics Engines -->
    <section class="space-y-2">
      <div class="flex items-center justify-between px-1">
        <h3 class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">Graphics Engines</h3>
        {#if runtimeStatus?.is_installed && (!runtimeStatus?.has_dxvk || !runtimeStatus?.has_winemetalgl)}
          <button
            onclick={onInstallEnginesRequest || onInstallRequest}
            class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] text-zinc-200 text-[11px] font-medium border border-white/[0.08] transition-colors cursor-pointer"
          >
            <Download class="w-3 h-3 text-zinc-400" />
            <span>Install Engines</span>
          </button>
        {/if}
      </div>

      <div class="rounded-xl border border-white/[0.07] bg-white/[0.025] divide-y divide-white/[0.06] text-xs">
        <div class="flex items-center justify-between px-4 py-3">
          <div class="flex items-center gap-2.5">
            <Layers class="w-3.5 h-3.5 text-zinc-400 shrink-0" />
            <div>
              <span class="text-zinc-200 font-medium">DXVK</span>
              <span class="text-zinc-500 text-[11px] font-mono ml-1.5">DirectX 9–11 via Vulkan</span>
            </div>
          </div>
          {#if runtimeStatus?.has_dxvk}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              <CheckCircle2 class="w-3 h-3" />
              Ready
            </span>
          {:else}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20">
              Not Installed
            </span>
          {/if}
        </div>

        <div class="flex items-center justify-between px-4 py-3">
          <div class="flex items-center gap-2.5">
            <ShieldCheck class="w-3.5 h-3.5 text-zinc-400 shrink-0" />
            <div>
              <span class="text-zinc-200 font-medium">Apple D3DMetal</span>
              <span class="text-zinc-500 text-[11px] font-mono ml-1.5">DirectX 11/12 (GPTK)</span>
            </div>
          </div>
          {#if runtimeStatus?.has_d3dmetal}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              <CheckCircle2 class="w-3 h-3" />
              Ready
            </span>
          {:else}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20">
              Not Installed
            </span>
          {/if}
        </div>

        <div class="flex items-center justify-between px-4 py-3">
          <div class="flex items-center gap-2.5">
            <Database class="w-3.5 h-3.5 text-zinc-400 shrink-0" />
            <div>
              <span class="text-zinc-200 font-medium">WineMetalGL</span>
              <span class="text-zinc-500 text-[11px] font-mono ml-1.5">OpenGL 1.0–4.6 via Metal</span>
            </div>
          </div>
          {#if runtimeStatus?.has_winemetalgl}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              <CheckCircle2 class="w-3 h-3" />
              Ready
            </span>
          {:else}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20">
              Not Installed
            </span>
          {/if}
        </div>
      </div>
    </section>

    <!-- Section 3: Windows Dependencies -->
    <section class="space-y-2">
      <div class="px-1">
        <h3 class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">Windows Dependencies</h3>
      </div>

      <div class="rounded-xl border border-white/[0.07] bg-white/[0.025] divide-y divide-white/[0.06] text-xs">
        <div class="flex items-center justify-between px-4 py-3 gap-3">
          <div class="flex items-center gap-2.5 min-w-0">
            <ShieldCheck class="w-3.5 h-3.5 text-zinc-400 shrink-0" />
            <div class="min-w-0">
              <span class="text-zinc-200 font-medium">MS Visual C++ 2015–2022</span>
              <span class="text-zinc-500 text-[11px] font-mono ml-1.5">vcruntime140</span>
            </div>
          </div>
          {#if runtimeStatus?.has_vcredist}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0">
              <CheckCircle2 class="w-3 h-3" />
              Installed
            </span>
          {:else}
            <button
              onclick={onInstallVcRedistRequest}
              disabled={!runtimeStatus?.is_installed}
              class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] disabled:opacity-40 text-zinc-200 text-[11px] font-medium border border-white/[0.08] transition-colors cursor-pointer shrink-0"
            >
              <Download class="w-3 h-3 text-zinc-400" />
              <span>Install VC++ (~24 MB)</span>
            </button>
          {/if}
        </div>

        <div class="flex items-center justify-between px-4 py-3 gap-3">
          <div class="flex items-center gap-2.5 min-w-0">
            <Database class="w-3.5 h-3.5 text-zinc-400 shrink-0" />
            <div class="min-w-0">
              <span class="text-zinc-200 font-medium">Windows Core Fonts</span>
              <span class="text-zinc-500 text-[11px] font-mono ml-1.5">Arial, Tahoma, Verdana</span>
            </div>
          </div>
          {#if runtimeStatus?.has_core_fonts}
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0">
              <CheckCircle2 class="w-3 h-3" />
              Synced
            </span>
          {:else}
            <button
              onclick={onSyncFontsRequest}
              class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] text-zinc-200 text-[11px] font-medium border border-white/[0.08] transition-colors cursor-pointer shrink-0"
            >
              <RefreshCw class="w-3 h-3 text-zinc-400" />
              <span>Sync Fonts</span>
            </button>
          {/if}
        </div>
      </div>
    </section>

    <!-- Section 4: Storage & Maintenance -->
    <section class="space-y-2">
      <div class="px-1">
        <h3 class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">Storage & Maintenance</h3>
      </div>

      <div class="rounded-xl border border-white/[0.07] bg-white/[0.025] divide-y divide-white/[0.06] text-xs">
        <div class="flex items-center justify-between px-4 py-3 gap-3">
          <div class="min-w-0 space-y-0.5">
            <span class="text-zinc-200 font-medium block">Shared Virtual Drive C:</span>
            <p class="text-[11px] text-zinc-500 truncate">Program Files, system32, and shared Windows runtime files</p>
          </div>
          <button
            onclick={openSharedDriveC}
            class="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] text-zinc-200 text-xs font-medium border border-white/[0.08] transition-colors cursor-pointer whitespace-nowrap shrink-0"
          >
            <HardDrive class="w-3.5 h-3.5 text-zinc-400" />
            <span>Open Drive C:</span>
          </button>
        </div>

        <div class="flex items-center justify-between px-4 py-3 gap-3">
          <div class="min-w-0 space-y-0.5">
            <span class="text-zinc-200 font-medium block">AppData & Micro-Prefixes</span>
            <p class="text-[11px] text-zinc-500 truncate">Isolated application data, user saves, and registry hives</p>
          </div>
          <button
            onclick={openPrefixesDir}
            class="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] text-zinc-200 text-xs font-medium border border-white/[0.08] transition-colors cursor-pointer whitespace-nowrap shrink-0"
          >
            <FolderArchive class="w-3.5 h-3.5 text-zinc-400" />
            <span>Open Prefixes</span>
          </button>
        </div>

        <div class="flex items-center justify-between px-4 py-3 gap-3">
          <div class="min-w-0 space-y-0.5">
            <div class="text-zinc-200 font-medium">Clean Runtimes & Cache</div>
            <p class="text-[11px] text-zinc-500 truncate">Remove downloaded runtimes and shared caches to free disk space</p>
          </div>
          <button
            onclick={() => (isConfirmRemoveAllOpen = true)}
            class="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium text-rose-400 hover:text-rose-300 bg-rose-500/10 hover:bg-rose-500/15 border border-rose-500/20 transition-colors cursor-pointer shrink-0"
          >
            <Trash2 class="w-3.5 h-3.5" />
            <span>Remove All</span>
          </button>
        </div>
      </div>
    </section>
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
