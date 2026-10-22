<script lang="ts">
  import { onMount } from 'svelte';
  import {
    inspectTarget,
    packToApp,
    dryRunCheck,
    launchInstalledApp,
    openInFinder,
    checkRuntimeStatus,
    installSwifterRuntime,
    type SwifterInspection,
    type RuntimeStatus
  } from '$lib/tauri-bridge';
  import Dropzone from '../components/Dropzone.svelte';
  import GameInspector from '../components/GameInspector.svelte';
  import ProgressModal from '../components/ProgressModal.svelte';
  import Shelf from '../components/Shelf.svelte';
  import Guide from '../components/Guide.svelte';
  import About from '../components/About.svelte';
  import {
    Package,
    Library,
    BookOpen,
    Info,
    Terminal,
    X,
    AlertCircle,
    Download,
    CheckCircle2,
    Cpu
  } from 'lucide-svelte';

  type ActiveTab = 'packer' | 'shelf' | 'guide' | 'about';

  let activeTab = $state<ActiveTab>('packer');
  let inspection = $state<SwifterInspection | null>(null);
  let isAnalyzing = $state(false);
  let isPacking = $state(false);
  let isDryRunning = $state(false);
  let errorMessage = $state('');

  // Runtime status state
  let runtimeStatus = $state<RuntimeStatus | null>(null);
  let isInstallingRuntime = $state(false);
  let isRuntimeModalOpen = $state(false);
  let runtimeInstallSuccess = $state(false);
  let runtimeInstallMsg = $state('');

  // Progress modal state
  let isProgressModalOpen = $state(false);
  let isProgressComplete = $state(false);
  let currentProgressStep = $state(1);
  let createdBundlePath = $state('');
  let packedAppName = $state('');

  // Dry-run drawer state
  let isDryRunDrawerOpen = $state(false);
  let dryRunLogs = $state<string[]>([]);

  async function refreshRuntimeStatus() {
    try {
      runtimeStatus = await checkRuntimeStatus();
    } catch {
      runtimeStatus = {
        is_installed: false,
        runtime_name: null,
        runtime_path: null,
        has_dxvk: false,
        has_d3dmetal: false,
      };
    }
  }

  onMount(() => {
    refreshRuntimeStatus();
  });

  async function handleInstallRuntime() {
    isInstallingRuntime = true;
    isRuntimeModalOpen = true;
    runtimeInstallSuccess = false;
    runtimeInstallMsg = '';
    try {
      const msg = await installSwifterRuntime();
      runtimeInstallMsg = msg;
      runtimeInstallSuccess = true;
      await refreshRuntimeStatus();
    } catch (err: unknown) {
      runtimeInstallMsg = err instanceof Error ? err.message : String(err);
      runtimeInstallSuccess = false;
    } finally {
      isInstallingRuntime = false;
    }
  }

  async function handleInspectPath(path: string) {
    isAnalyzing = true;
    errorMessage = '';
    try {
      const res = await inspectTarget(path);
      inspection = res;
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : String(err);
      inspection = null;
    } finally {
      isAnalyzing = false;
    }
  }

  async function handlePack(options: {
    appName: string;
    backend: string;
    enableMsync: boolean;
    copyFiles: boolean;
  }) {
    if (!inspection) return;

    isPacking = true;
    errorMessage = '';
    isProgressModalOpen = true;
    isProgressComplete = false;
    currentProgressStep = 1;
    packedAppName = options.appName;

    try {
      // Step 1: Reading binary & assets
      currentProgressStep = 1;
      await new Promise((r) => setTimeout(r, 400));

      // Step 2: Micro-prefix
      currentProgressStep = 2;
      await new Promise((r) => setTimeout(r, 400));

      // Step 3: Launcher
      currentProgressStep = 3;
      await new Promise((r) => setTimeout(r, 300));

      // Step 4: Bundle builder
      currentProgressStep = 4;
      const bundle = await packToApp({
        game_folder_path: inspection.game_folder_path,
        target_exe_name: inspection.target_exe_name,
        app_name: options.appName,
        backend_choice: options.backend,
        enable_msync: options.enableMsync,
        copy_files_instead_of_symlink: options.copyFiles,
      });

      createdBundlePath = bundle;
      isProgressComplete = true;
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : String(err);
      isProgressModalOpen = false;
    } finally {
      isPacking = false;
    }
  }

  async function handleTestRun(exePath: string, backend: string) {
    isDryRunning = true;
    errorMessage = '';
    try {
      const logs = await dryRunCheck(exePath, backend);
      dryRunLogs = logs;
      isDryRunDrawerOpen = true;
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : String(err);
    } finally {
      isDryRunning = false;
    }
  }

  function handleResetPacker() {
    inspection = null;
    errorMessage = '';
  }
</script>

<div class="flex-1 flex flex-col h-full overflow-hidden bg-[#111113] text-zinc-200">
  <!-- Segmented Navigation Header -->
  <div class="px-8 pt-4 pb-2 flex items-center justify-between shrink-0">
    <div class="flex items-center p-1 rounded-xl glass-card space-x-1 border border-zinc-800 bg-zinc-900/40">
      <button
        onclick={() => (activeTab = 'packer')}
        class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all {activeTab === 'packer' ? 'bg-zinc-800 text-zinc-100 border border-zinc-700 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Package class="w-3.5 h-3.5 text-zinc-400" />
        Packer
      </button>

      <button
        onclick={() => (activeTab = 'shelf')}
        class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all {activeTab === 'shelf' ? 'bg-zinc-800 text-zinc-100 border border-zinc-700 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Library class="w-3.5 h-3.5 text-zinc-400" />
        Shelf
      </button>

      <button
        onclick={() => (activeTab = 'guide')}
        class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all {activeTab === 'guide' ? 'bg-zinc-800 text-zinc-100 border border-zinc-700 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <BookOpen class="w-3.5 h-3.5 text-zinc-400" />
        Guide
      </button>

      <button
        onclick={() => (activeTab = 'about')}
        class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all {activeTab === 'about' ? 'bg-zinc-800 text-zinc-100 border border-zinc-700 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Info class="w-3.5 h-3.5 text-zinc-400" />
        About
      </button>
    </div>

    <!-- Right Header Info: Runtime Status & Version -->
    <div class="flex items-center gap-3">
      {#if runtimeStatus}
        {#if runtimeStatus.is_installed}
          <div class="hidden sm:inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-900 border border-zinc-800 text-[10px] text-zinc-300 font-mono">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
            {runtimeStatus.runtime_name || 'Wine Ready'}
          </div>
        {:else}
          <button
            onclick={handleInstallRuntime}
            class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-[10px] text-zinc-300 transition-all active:scale-95"
          >
            <Download class="w-3 h-3 text-zinc-400" />
            <span>Install Wine Runtime</span>
          </button>
        {/if}
      {/if}

      <div class="text-[11px] text-zinc-500 font-mono">
        Swifter v0.1 • by msandypr
      </div>
    </div>
  </div>

  <!-- Runtime Warning Banner (If Not Installed) -->
  {#if runtimeStatus && !runtimeStatus.is_installed && activeTab === 'packer'}
    <div class="mx-8 mt-2 p-3.5 rounded-2xl bg-zinc-900/90 border border-zinc-800 flex items-center justify-between text-xs space-x-3">
      <div class="flex items-center gap-2.5 text-zinc-300">
        <Cpu class="w-4 h-4 text-zinc-400 shrink-0" />
        <span>No Wine translation runtime detected on this Mac. Download Swifter's portable runtime to run Windows games.</span>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <button
          onclick={handleInstallRuntime}
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 transition-all shadow-sm"
        >
          <Download class="w-3.5 h-3.5" />
          Download Runtime (1-Click)
        </button>
        <button
          onclick={() => (activeTab = 'guide')}
          class="px-2.5 py-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs border border-zinc-700 transition-all"
        >
          Guide
        </button>
      </div>
    </div>
  {/if}

  <!-- Global Error Banner -->
  {#if errorMessage}
    <div class="mx-8 mt-2 p-3 rounded-xl bg-zinc-900 border border-rose-500/40 flex items-center justify-between text-rose-300 text-xs">
      <div class="flex items-center gap-2">
        <AlertCircle class="w-4 h-4 text-rose-400 shrink-0" />
        <span>{errorMessage}</span>
      </div>
      <button onclick={() => (errorMessage = '')} class="p-1 hover:bg-zinc-800 rounded">
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  {/if}

  <!-- Tab Content Views -->
  <div class="flex-1 overflow-hidden relative flex flex-col items-center justify-center">
    {#if activeTab === 'packer'}
      {#if !inspection}
        <Dropzone onSelectPath={handleInspectPath} {isAnalyzing} />
      {:else}
        <GameInspector
          {inspection}
          onPack={handlePack}
          onTestRun={handleTestRun}
          onBack={handleResetPacker}
          {isPacking}
          {isDryRunning}
        />
      {/if}
    {:else if activeTab === 'shelf'}
      <Shelf onNavigatePacker={() => (activeTab = 'packer')} />
    {:else if activeTab === 'guide'}
      <Guide />
    {:else if activeTab === 'about'}
      <About />
    {/if}
  </div>

  <!-- Runtime Downloader Modal -->
  {#if isRuntimeModalOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md">
      <div class="w-full max-w-md glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-200">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-200">
              {#if isInstallingRuntime}
                <div class="w-4 h-4 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
              {:else if runtimeInstallSuccess}
                <CheckCircle2 class="w-5 h-5 text-zinc-100" />
              {:else}
                <AlertCircle class="w-5 h-5 text-rose-400" />
              {/if}
            </div>
            <div>
              <h3 class="text-sm font-semibold text-zinc-100">
                {isInstallingRuntime ? 'Installing Wine Runtime...' : runtimeInstallSuccess ? 'Runtime Ready' : 'Installation Notice'}
              </h3>
              <p class="text-xs text-zinc-400">
                {isInstallingRuntime ? 'Downloading and extracting portable runtime for macOS.' : runtimeInstallSuccess ? 'Swifter can now run all Windows applications.' : 'Download failed.'}
              </p>
            </div>
          </div>

          {#if !isInstallingRuntime}
            <button onclick={() => (isRuntimeModalOpen = false)} class="text-zinc-400 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800 transition-colors">
              <X class="w-4 h-4" />
            </button>
          {/if}
        </div>

        {#if isInstallingRuntime}
          <div class="space-y-2 text-xs text-zinc-400">
            <div class="p-3 rounded-xl bg-zinc-950 border border-zinc-800 font-mono text-[11px] text-zinc-400 space-y-1">
              <div>→ Downloading official Wine runtime archive from GitHub releases...</div>
              <div>→ Unpacking into Swifter shared runtimes folder...</div>
              <div>→ Setting up POSIX permissions...</div>
            </div>
            <p class="text-[11px] text-zinc-500 text-center">This runs once and will be shared across all your packaged games.</p>
          </div>
        {:else if runtimeInstallSuccess}
          <div class="p-3 rounded-xl bg-zinc-900 border border-zinc-800 space-y-1 text-xs text-zinc-300">
            <div class="font-medium text-zinc-200">Installation Successful</div>
            <p class="text-[11px] text-zinc-400">{runtimeInstallMsg}</p>
          </div>
          <button
            onclick={() => (isRuntimeModalOpen = false)}
            class="w-full py-2.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-semibold active:scale-95 transition-all shadow-sm"
          >
            Done
          </button>
        {:else}
          <div class="p-3 rounded-xl bg-zinc-900 border border-rose-500/30 text-xs text-rose-300 space-y-1">
            <div class="font-medium">Installation Failed</div>
            <p class="text-[11px] text-rose-400">{runtimeInstallMsg}</p>
            <p class="text-[11px] text-zinc-400 pt-1">You can also install Wine via Homebrew: <code class="text-zinc-200 bg-zinc-800 px-1 py-0.5 rounded">brew install --cask wine-stable</code> or install Whisky.</p>
          </div>
          <button
            onclick={() => (isRuntimeModalOpen = false)}
            class="w-full py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium transition-all"
          >
            Close
          </button>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Packaging Progress Modal -->
  <ProgressModal
    isOpen={isProgressModalOpen}
    isComplete={isProgressComplete}
    bundlePath={createdBundlePath}
    appName={packedAppName}
    currentStep={currentProgressStep}
    onClose={() => {
      isProgressModalOpen = false;
      if (isProgressComplete) {
        activeTab = 'shelf';
        handleResetPacker();
      }
    }}
    onLaunch={(path) => launchInstalledApp(path)}
    onReveal={(path) => openInFinder(path)}
  />

  <!-- Dry-Run Output Drawer/Modal -->
  {#if isDryRunDrawerOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/60 backdrop-blur-md">
      <div class="w-full max-w-2xl h-[420px] glass-panel rounded-2xl p-5 shadow-2xl flex flex-col space-y-3 border border-zinc-800">
        <div class="flex items-center justify-between pb-2 border-b border-zinc-800">
          <div class="flex items-center gap-2 text-xs font-semibold text-zinc-200">
            <Terminal class="w-4 h-4 text-zinc-400" />
            Wine 3-Second Dry-Run Diagnostic
          </div>
          <button
            onclick={() => (isDryRunDrawerOpen = false)}
            class="p-1 text-zinc-400 hover:text-zinc-200 rounded-lg hover:bg-zinc-800 transition-colors"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="flex-1 overflow-y-auto font-mono text-[11px] p-3 rounded-xl bg-zinc-950 border border-zinc-800 text-zinc-300 space-y-1 select-text">
          {#each dryRunLogs as line}
            <div class="{line.includes('err:') ? 'text-rose-400' : line.includes('warn:') ? 'text-amber-400' : 'text-zinc-300'}">
              {line}
            </div>
          {/each}
        </div>

        <div class="flex justify-end pt-1">
          <button
            onclick={() => (isDryRunDrawerOpen = false)}
            class="px-4 py-2 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-200 border border-zinc-700 transition-colors"
          >
            Close Logs
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
