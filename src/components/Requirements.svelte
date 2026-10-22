<script lang="ts">
  import type { RuntimeStatus, RuntimeDownloadProgress } from '$lib/tauri-bridge';
  import {
    Cpu,
    CheckCircle2,
    XCircle,
    Download,
    Trash2,
    Layers,
    ShieldCheck,
    AlertTriangle,
    RefreshCw,
    Terminal,
    HardDrive
  } from 'lucide-svelte';

  let {
    runtimeStatus,
    onInstallRequest,
    onRemoveRequest,
    onRefresh
  }: {
    runtimeStatus: RuntimeStatus | null;
    onInstallRequest: () => void;
    onRemoveRequest: () => void;
    onRefresh: () => void;
  } = $props();

  let isRefreshing = $state(false);

  async function handleRefreshClick() {
    isRefreshing = true;
    await onRefresh();
    setTimeout(() => {
      isRefreshing = false;
    }, 400);
  }
</script>

<div class="w-full max-w-3xl mx-auto flex flex-col p-6 space-y-6 overflow-y-auto max-h-[calc(100vh-4rem)] text-zinc-300">
  <!-- Header with Refresh Button -->
  <div class="flex items-center justify-between border-b border-zinc-800 pb-4">
    <div>
      <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
        <Cpu class="w-4 h-4 text-zinc-400" />
        System Requirements & Runtimes
      </h2>
      <p class="text-xs text-zinc-400 mt-0.5">
        Manage underlying compatibility engines needed to run Windows games on macOS.
      </p>
    </div>

    <button
      onclick={handleRefreshClick}
      disabled={isRefreshing}
      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-zinc-800 bg-zinc-900/60 hover:bg-zinc-800 text-xs font-medium text-zinc-300 transition-all active:scale-95 disabled:opacity-50"
    >
      <RefreshCw class="w-3.5 h-3.5 {isRefreshing ? 'animate-spin' : ''}" />
      Check Status
    </button>
  </div>

  <div class="space-y-4">
    <!-- 1. Wine Core Runtime Card -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-4">
      <div class="flex items-start justify-between gap-4">
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <h3 class="text-sm font-semibold text-zinc-100">Wine Core Translation Layer</h3>
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
          <p class="text-xs text-zinc-400 leading-relaxed">
            Essential Windows OS-to-macOS binary translation engine. Windows .exe executables cannot run on macOS without this.
          </p>
        </div>
      </div>

      <!-- Runtime Details or Install Callout -->
      {#if runtimeStatus?.is_installed}
        <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-2 text-xs">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px]">
            <div>
              <span class="text-zinc-500">Provider:</span>
              <span class="text-zinc-200 font-medium ml-1.5">{runtimeStatus.runtime_name || 'Detected Runtime'}</span>
            </div>
            <div class="truncate">
              <span class="text-zinc-500">Binary Path:</span>
              <span class="text-zinc-400 font-mono ml-1.5" title={runtimeStatus.runtime_path || ''}>{runtimeStatus.runtime_path || '-'}</span>
            </div>
          </div>

          <!-- If installed via Swifter Internal, offer remove button -->
          {#if runtimeStatus.runtime_name?.includes('Swifter')}
            <div class="pt-2 border-t border-zinc-800/80 flex items-center justify-between">
              <span class="text-[11px] text-zinc-500">Installed in Swifter Application Support</span>
              <button
                onclick={onRemoveRequest}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-rose-400 hover:text-rose-300 hover:bg-rose-500/10 border border-rose-500/20 transition-all active:scale-95"
              >
                <Trash2 class="w-3.5 h-3.5" />
                Remove Runtime
              </button>
            </div>
          {/if}
        </div>
      {:else}
        <div class="p-4 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-3">
          <div class="flex items-start gap-2.5 text-xs text-zinc-300">
            <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
            <div class="space-y-1">
              <div class="font-medium text-zinc-200">Wine runtime is missing</div>
              <p class="text-[11px] text-zinc-400 leading-relaxed">
                You can download Swifter's official pre-compiled portable runtime (~182 MB), or install via Homebrew in one line.
              </p>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2.5 pt-1">
            <button
              onclick={onInstallRequest}
              class="inline-flex items-center gap-1.5 px-4 py-2 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 transition-all shadow-sm"
            >
              <Download class="w-3.5 h-3.5" />
              Install Swifter Runtime (~182 MB)
            </button>

            <span class="text-xs text-zinc-500">or via Homebrew:</span>
            <code class="px-2.5 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-[11px] text-zinc-300 font-mono select-all">
              brew install --cask wine-stable
            </code>
          </div>
        </div>
      {/if}
    </div>

    <!-- 2. DXVK Translation Layer Card -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Layers class="w-4 h-4 text-zinc-400" />
          <h3 class="text-sm font-semibold text-zinc-100">DXVK Engine (DirectX 9–11 via MoltenVK)</h3>
        </div>
        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
          <CheckCircle2 class="w-3 h-3" />
          Included & Ready
        </span>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-6">
        Translates DirectX 9, 10, and 11 draw calls into native Vulkan shaders via MoltenVK on macOS. Automatically managed by Swifter's micro-prefix DLL overrides.
      </p>
    </div>

    <!-- 3. Apple D3DMetal Card -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <ShieldCheck class="w-4 h-4 text-zinc-400" />
          <h3 class="text-sm font-semibold text-zinc-100">Apple D3DMetal (Game Porting Toolkit 2)</h3>
        </div>
        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
          <CheckCircle2 class="w-3 h-3" />
          Supported (Metal 3)
        </span>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-6">
        Apple's native translation engine translating DirectX 11/12 directly into Metal shading language with ESYNC and MSYNC synchronization support.
      </p>
    </div>
  </div>
</div>
