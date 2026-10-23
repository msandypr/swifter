<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import {
    inspectTarget,
    packToApp,
    dryRunCheck,
    launchInstalledApp,
    openInFinder,
    checkRuntimeStatus,
    installSwifterRuntime,
    removeSwifterRuntime,
    type SwifterInspection,
    type RuntimeStatus,
    type RuntimeDownloadProgress
  } from '$lib/tauri-bridge';
  import Dropzone from '../components/Dropzone.svelte';
  import GameInspector from '../components/GameInspector.svelte';
  import ProgressModal from '../components/ProgressModal.svelte';
  import Shelf from '../components/Shelf.svelte';
  import SettingsView from '../components/Settings.svelte';
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
    Settings,
    Trash2,
    ShieldAlert,
    AlertTriangle,
    ChevronDown,
    ChevronUp
  } from 'lucide-svelte';

  type ActiveTab = 'packer' | 'shelf' | 'settings' | 'guide' | 'about';

  let activeTab = $state<ActiveTab>('packer');
  let inspection = $state<SwifterInspection | null>(null);
  let isAnalyzing = $state(false);
  let isPacking = $state(false);
  let isDryRunning = $state(false);
  let errorMessage = $state('');

  // Runtime status & progress state
  let runtimeStatus = $state<RuntimeStatus | null>(null);
  let isInstallingRuntime = $state(false);
  let isRuntimeModalOpen = $state(false);
  let runtimeInstallSuccess = $state(false);
  let runtimeInstallMsg = $state('');
  let runtimeProgress = $state<RuntimeDownloadProgress | null>(null);

  // User confirmation dialog states
  let isConfirmInstallOpen = $state(false);
  let isConfirmRemoveOpen = $state(false);
  let isRemovingRuntime = $state(false);

  // Progress modal state
  let isProgressModalOpen = $state(false);
  let isProgressComplete = $state(false);
  let currentProgressStep = $state(1);
  let createdBundlePath = $state('');
  let packedAppName = $state('');
  let packError = $state('');

  // Dry-run modal state
  let isDryRunDrawerOpen = $state(false);
  let dryRunLogs = $state<string[]>([]);
  let isShowTerminalLogs = $state(false);

  let dryRunHasError = $derived(
    dryRunLogs.some(
      (l) =>
        l.toLowerCase().includes('failed to spawn') ||
        l.toLowerCase().includes('wine error') ||
        l.toLowerCase().includes('c0000135')
    )
  );

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

    let unlisten: (() => void) | undefined;
    listen<RuntimeDownloadProgress>('runtime-download-progress', (event) => {
      runtimeProgress = event.payload;
      if (event.payload.stage === 'completed') {
        runtimeInstallSuccess = true;
        runtimeInstallMsg = event.payload.message;
        refreshRuntimeStatus();
      } else if (event.payload.stage === 'error') {
        runtimeInstallSuccess = false;
        runtimeInstallMsg = event.payload.message;
      }
    }).then((unsub) => {
      unlisten = unsub;
    });

    return () => {
      if (unlisten) unlisten();
    };
  });

  function requestInstallRuntime() {
    isConfirmInstallOpen = true;
  }

  function requestRemoveRuntime() {
    isConfirmRemoveOpen = true;
  }

  async function confirmAndStartInstall() {
    isConfirmInstallOpen = false;
    isInstallingRuntime = true;
    isRuntimeModalOpen = true;
    runtimeInstallSuccess = false;
    runtimeInstallMsg = '';
    runtimeProgress = {
      stage: 'downloading',
      percent: 0,
      downloaded_mb: 0,
      total_mb: 182.1,
      speed_mb: 0,
      message: 'Connecting to GitHub releases...',
    };

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

  async function confirmAndStartRemove() {
    isConfirmRemoveOpen = false;
    isRemovingRuntime = true;
    errorMessage = '';
    try {
      await removeSwifterRuntime();
      await refreshRuntimeStatus();
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : String(err);
    } finally {
      isRemovingRuntime = false;
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
    targetExe?: string;
    backend: string;
    enableMsync: boolean;
    copyFiles: boolean;
  }) {
    if (!inspection) return;

    isPacking = true;
    errorMessage = '';
    packError = '';
    isProgressModalOpen = true;
    isProgressComplete = false;
    currentProgressStep = 1;
    packedAppName = options.appName;

    try {
      currentProgressStep = 1;
      await new Promise((r) => setTimeout(r, 400));

      currentProgressStep = 2;
      await new Promise((r) => setTimeout(r, 400));

      currentProgressStep = 3;
      await new Promise((r) => setTimeout(r, 300));

      currentProgressStep = 4;
      const bundle = await packToApp({
        game_folder_path: inspection.game_folder_path,
        target_exe_name: options.targetExe || inspection.target_exe_name,
        app_name: options.appName,
        backend_choice: options.backend,
        enable_msync: options.enableMsync,
        copy_files_instead_of_symlink: options.copyFiles,
      });

      createdBundlePath = bundle;
      isProgressComplete = true;
    } catch (err: unknown) {
      packError = err instanceof Error ? err.message : String(err);
      isProgressComplete = false;
    } finally {
      isPacking = false;
    }
  }

  async function handleTestRun(exePath: string, backend: string) {
    isDryRunning = true;
    errorMessage = '';
    isShowTerminalLogs = false;
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
  <!-- Segmented Navigation Header with Centered Navbar & Integrated Logo -->
  <div class="px-8 pt-4 pb-2 flex items-center justify-center shrink-0">
    <div class="flex items-center p-1 rounded-xl glass-card border border-zinc-800 bg-zinc-900/60 shadow-sm">
      <!-- Integrated Brand Header -->
      <div class="flex items-center gap-2 pl-2 pr-3 py-1 border-r border-zinc-800 mr-1 select-none">
        <img
          src="/logo-swifter.png"
          alt="Swifter"
          class="w-5 h-5 rounded object-contain"
        />
        <span class="text-xs font-semibold text-zinc-100 tracking-tight">Swifter</span>
      </div>

      <!-- Segmented Buttons with 0px Geometry Shift -->
      <div class="flex items-center space-x-1">
        <button
          onclick={() => (activeTab = 'packer')}
          class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium border transition-colors duration-150 ease-out cursor-pointer {activeTab === 'packer' ? 'bg-zinc-800 text-zinc-100 border-zinc-700 shadow-xs' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'}"
        >
          <Package class="w-3.5 h-3.5 text-zinc-400" />
          Packer
        </button>

        <button
          onclick={() => (activeTab = 'shelf')}
          class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium border transition-colors duration-150 ease-out cursor-pointer {activeTab === 'shelf' ? 'bg-zinc-800 text-zinc-100 border-zinc-700 shadow-xs' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'}"
        >
          <Library class="w-3.5 h-3.5 text-zinc-400" />
          Shelf
        </button>

        <!-- Settings Tab with live status dot -->
        <button
          onclick={() => (activeTab = 'settings')}
          class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium border transition-colors duration-150 ease-out cursor-pointer {activeTab === 'settings' ? 'bg-zinc-800 text-zinc-100 border-zinc-700 shadow-xs' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'}"
        >
          <Settings class="w-3.5 h-3.5 text-zinc-400" />
          <span>Settings</span>
          {#if runtimeStatus}
            {#if runtimeStatus.is_installed}
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.7)]"></span>
            {:else}
              <span class="w-1.5 h-1.5 rounded-full bg-rose-500 shadow-[0_0_6px_rgba(244,63,94,0.7)] animate-pulse"></span>
            {/if}
          {/if}
        </button>

        <button
          onclick={() => (activeTab = 'guide')}
          class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium border transition-colors duration-150 ease-out cursor-pointer {activeTab === 'guide' ? 'bg-zinc-800 text-zinc-100 border-zinc-700 shadow-xs' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'}"
        >
          <BookOpen class="w-3.5 h-3.5 text-zinc-400" />
          Guide
        </button>

        <button
          onclick={() => (activeTab = 'about')}
          class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-medium border transition-colors duration-150 ease-out cursor-pointer {activeTab === 'about' ? 'bg-zinc-800 text-zinc-100 border-zinc-700 shadow-xs' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'}"
        >
          <Info class="w-3.5 h-3.5 text-zinc-400" />
          About
        </button>
      </div>
    </div>
  </div>

  <!-- Runtime Notice Banner (If Wine Not Installed) -->
  {#if runtimeStatus && !runtimeStatus.is_installed && activeTab === 'packer'}
    <div class="mx-8 mt-2 p-3.5 rounded-2xl bg-zinc-900/90 border border-zinc-800 flex items-center justify-between text-xs space-x-3">
      <div class="flex items-center gap-2.5 text-zinc-300">
        <AlertTriangle class="w-4 h-4 text-zinc-400 shrink-0" />
        <span>No Wine translation runtime detected on this Mac. Configure or download it in Settings.</span>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <button
          onclick={requestInstallRuntime}
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 transition-all shadow-sm cursor-pointer"
        >
          <Download class="w-3.5 h-3.5" />
          Download Runtime
        </button>
        <button
          onclick={() => (activeTab = 'settings')}
          class="px-2.5 py-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs border border-zinc-700 transition-all cursor-pointer"
        >
          Settings
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

  <!-- Tab Content Views with Smooth Fade Transitions & Proper Top Alignment -->
  <div class="flex-1 overflow-y-auto relative flex flex-col items-center justify-start w-full px-6 py-4 transition-opacity duration-200">
    {#if activeTab === 'packer'}
      {#if !inspection}
        <div class="my-auto w-full flex items-center justify-center py-8">
          <Dropzone onSelectPath={handleInspectPath} {isAnalyzing} />
        </div>
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
    {:else if activeTab === 'settings'}
      <SettingsView
        {runtimeStatus}
        onInstallRequest={requestInstallRuntime}
        onRemoveRequest={requestRemoveRuntime}
        onRefresh={refreshRuntimeStatus}
      />
    {:else if activeTab === 'guide'}
      <Guide />
    {:else if activeTab === 'about'}
      <About />
    {/if}
  </div>

  <!-- User Agreement Confirmation Modal: Install Runtime -->
  {#if isConfirmInstallOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md">
      <div class="w-full max-w-md glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-start gap-3.5">
          <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-200 shrink-0">
            <Download class="w-5 h-5 text-zinc-300" />
          </div>
          <div class="space-y-1">
            <h3 class="text-sm font-semibold text-zinc-100">Install Wine Translation Runtime</h3>
            <p class="text-xs text-zinc-400 leading-relaxed">
              Swifter will download and configure the official portable Wine runtime for macOS (~182 MB from GitHub releases).
            </p>
          </div>
        </div>

        <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 text-xs space-y-2 text-zinc-300">
          <div class="flex justify-between text-[11px]">
            <span class="text-zinc-500">Download Size:</span>
            <span class="font-mono text-zinc-200">~182.1 MB</span>
          </div>
          <div class="flex justify-between text-[11px]">
            <span class="text-zinc-500">Install Location:</span>
            <span class="font-mono text-zinc-300 truncate max-w-[200px]" title="~/Library/Application Support/Swifter/runtimes/wine">~/Library/Application Support/Swifter/...</span>
          </div>
          <div class="flex justify-between text-[11px]">
            <span class="text-zinc-500">License:</span>
            <span class="text-zinc-300">LGPL v2.1+ (Open Source)</span>
          </div>
        </div>

        <div class="flex items-center gap-2 pt-1">
          <button
            onclick={confirmAndStartInstall}
            class="flex-1 py-2.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-semibold active:scale-95 transition-all shadow-sm cursor-pointer"
          >
            Confirm & Download
          </button>
          <button
            onclick={() => (isConfirmInstallOpen = false)}
            class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-all cursor-pointer"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- User Agreement Confirmation Modal: Remove Runtime -->
  {#if isConfirmRemoveOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md">
      <div class="w-full max-w-md glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-start gap-3.5">
          <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-rose-500/30 flex items-center justify-center text-rose-400 shrink-0">
            <Trash2 class="w-5 h-5" />
          </div>
          <div class="space-y-1">
            <h3 class="text-sm font-semibold text-zinc-100">Remove Swifter Wine Runtime</h3>
            <p class="text-xs text-zinc-400 leading-relaxed">
              Are you sure you want to delete the internal Wine runtime from your Mac?
            </p>
          </div>
        </div>

        <div class="p-3 rounded-xl bg-zinc-950/80 border border-zinc-800 text-xs text-zinc-400 leading-relaxed">
          Packaged games will not be able to launch unless an alternative runtime (such as Homebrew Wine or Whisky) is installed.
        </div>

        <div class="flex items-center gap-2 pt-1">
          <button
            onclick={confirmAndStartRemove}
            disabled={isRemovingRuntime}
            class="flex-1 py-2.5 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold active:scale-95 transition-all shadow-sm cursor-pointer disabled:opacity-50"
          >
            {isRemovingRuntime ? 'Removing...' : 'Yes, Delete Runtime'}
          </button>
          <button
            onclick={() => (isConfirmRemoveOpen = false)}
            class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-all cursor-pointer"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Runtime Downloader Modal with Real-Time Progress -->
  {#if isRuntimeModalOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md">
      <div class="w-full max-w-lg glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-200">
        <!-- Modal Header -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-200">
              {#if isInstallingRuntime}
                <div class="w-4 h-4 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
              {:else if runtimeInstallSuccess}
                <CheckCircle2 class="w-5 h-5 text-emerald-400" />
              {:else}
                <AlertCircle class="w-5 h-5 text-rose-400" />
              {/if}
            </div>
            <div>
              <h3 class="text-sm font-semibold text-zinc-100">
                {isInstallingRuntime ? 'Installing Wine Translation Runtime' : runtimeInstallSuccess ? 'Wine Runtime Ready' : 'Installation Notice'}
              </h3>
              <p class="text-xs text-zinc-400">
                {isInstallingRuntime ? 'Downloading and extracting portable package (~182 MB).' : runtimeInstallSuccess ? 'Swifter can now run all Windows applications.' : 'Download interrupted.'}
              </p>
            </div>
          </div>

          {#if !isInstallingRuntime}
            <button onclick={() => (isRuntimeModalOpen = false)} class="text-zinc-400 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer">
              <X class="w-4 h-4" />
            </button>
          {/if}
        </div>

        {#if isInstallingRuntime}
          <!-- Real-Time Progress Indicator -->
          <div class="space-y-3 p-4 rounded-xl bg-zinc-950/80 border border-zinc-800/80">
            <!-- Progress Bar -->
            <div class="space-y-1.5">
              <div class="flex items-center justify-between text-xs text-zinc-300">
                <span class="font-medium">
                  {runtimeProgress?.stage === 'downloading'
                    ? `Downloading package (${runtimeProgress.percent.toFixed(0)}%)`
                    : runtimeProgress?.stage === 'extracting'
                    ? 'Extracting archive...'
                    : 'Configuring permissions...'}
                </span>
                <span class="font-mono text-[11px] text-zinc-400">
                  {#if runtimeProgress && runtimeProgress.stage === 'downloading'}
                    {runtimeProgress.downloaded_mb.toFixed(1)} / {runtimeProgress.total_mb.toFixed(1)} MB
                  {:else if runtimeProgress}
                    100%
                  {/if}
                </span>
              </div>

              <!-- Bar Track -->
              <div class="w-full h-2 bg-zinc-800 rounded-full overflow-hidden">
                <div
                  class="h-full bg-zinc-200 rounded-full transition-all duration-300"
                  style="width: {runtimeProgress ? Math.max(runtimeProgress.percent, 3) : 3}%"
                ></div>
              </div>
            </div>

            <!-- Details Row: Speed & Status Message -->
            <div class="flex items-center justify-between text-[11px] text-zinc-500 pt-1">
              <span class="font-mono truncate pr-2">
                {runtimeProgress?.message || 'Initiating network connection...'}
              </span>
              {#if runtimeProgress && runtimeProgress.stage === 'downloading' && runtimeProgress.speed_mb > 0}
                <span class="font-mono text-zinc-400 shrink-0">{runtimeProgress.speed_mb.toFixed(1)} MB/s</span>
              {/if}
            </div>
          </div>

          <!-- Quick Homebrew Tip -->
          <div class="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800/60 space-y-1 text-xs">
            <span class="text-zinc-400 font-medium">Alternative (Instant via Homebrew):</span>
            <div class="font-mono text-[11px] text-zinc-300 bg-zinc-950 px-2 py-1 rounded border border-zinc-800/80 select-all">
              brew install --cask wine-stable
            </div>
            <p class="text-[10px] text-zinc-500">If you run this in your terminal, Swifter will immediately detect it.</p>
          </div>
        {:else if runtimeInstallSuccess}
          <!-- Success State -->
          <div class="p-4 rounded-xl bg-zinc-900 border border-zinc-800 space-y-2 text-xs text-zinc-300">
            <div class="font-semibold text-zinc-100 flex items-center gap-1.5">
              <CheckCircle2 class="w-4 h-4 text-emerald-400" />
              Runtime Verified & Ready
            </div>
            <p class="text-[11px] text-zinc-400 leading-relaxed">
              {runtimeInstallMsg}
            </p>
          </div>
          <button
            onclick={() => (isRuntimeModalOpen = false)}
            class="w-full py-2.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-semibold active:scale-95 transition-all shadow-sm cursor-pointer"
          >
            Done
          </button>
        {:else}
          <!-- Error State -->
          <div class="p-4 rounded-xl bg-zinc-900 border border-rose-500/30 text-xs text-rose-300 space-y-2">
            <div class="font-semibold text-rose-200">Download Failed</div>
            <p class="text-[11px] text-rose-300 leading-relaxed">{runtimeInstallMsg}</p>
            <div class="pt-2 border-t border-rose-500/20 text-zinc-300 space-y-1">
              <span class="text-zinc-400">You can also install Wine in 1 line via Homebrew:</span>
              <div class="font-mono text-[11px] bg-zinc-950 px-2 py-1 rounded border border-zinc-800 select-all">
                brew install --cask wine-stable
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              onclick={confirmAndStartInstall}
              class="flex-1 py-2.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-semibold transition-all shadow-sm cursor-pointer"
            >
              Retry Download
            </button>
            <button
              onclick={() => (isRuntimeModalOpen = false)}
              class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-all cursor-pointer"
            >
              Close
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Packaging Progress Modal -->
  <ProgressModal
    isOpen={isProgressModalOpen}
    isComplete={isProgressComplete}
    error={packError}
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

  <!-- User-Friendly Dry-Run Diagnostic Modal -->
  {#if isDryRunDrawerOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md">
      <div class="w-full max-w-lg glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-150">
        <!-- Header -->
        <div class="flex items-center justify-between pb-2 border-b border-zinc-800">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-300 shrink-0">
              {#if dryRunHasError}
                <AlertTriangle class="w-5 h-5 text-rose-400" />
              {:else}
                <CheckCircle2 class="w-5 h-5 text-emerald-400" />
              {/if}
            </div>
            <div>
              <h3 class="text-sm font-semibold text-zinc-100">
                {dryRunHasError ? 'Test Run Encountered An Issue' : 'Test Run Successful'}
              </h3>
              <p class="text-xs text-zinc-400">
                {dryRunHasError ? 'Could not execute the binary smoothly.' : 'The game launched properly and is ready to package.'}
              </p>
            </div>
          </div>

          <button
            onclick={() => (isDryRunDrawerOpen = false)}
            class="p-1 text-zinc-400 hover:text-zinc-200 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Result Summary -->
        <div class="p-3.5 rounded-xl bg-zinc-950/80 border border-zinc-800 text-xs space-y-2 text-zinc-300">
          {#if dryRunHasError}
            <div class="text-rose-300 leading-relaxed font-medium">
              The game process could not start or a required dependency was not found.
            </div>
            <p class="text-[11px] text-zinc-400 leading-relaxed">
              Tip: If this is an installer, run the setup wizard first. You can also try changing the translation engine (DXVK vs Vanilla) in the settings above.
            </p>
          {:else}
            <div class="text-zinc-200 leading-relaxed font-medium">
              Process initialized and ran without crashing.
            </div>
            <p class="text-[11px] text-zinc-400 leading-relaxed">
              The translation layer successfully allocated memory, created windows, and handled graphics initialization.
            </p>
          {/if}
        </div>

        <!-- Collapsible Technical Terminal Logs -->
        <div class="space-y-2">
          <button
            onclick={() => (isShowTerminalLogs = !isShowTerminalLogs)}
            class="inline-flex items-center gap-1.5 text-[11px] text-zinc-500 hover:text-zinc-300 transition-colors cursor-pointer"
          >
            {#if isShowTerminalLogs}
              <ChevronUp class="w-3.5 h-3.5" />
              <span>Hide Technical Details</span>
            {:else}
              <ChevronDown class="w-3.5 h-3.5" />
              <span>View Technical Details (Logs)</span>
            {/if}
          </button>

          {#if isShowTerminalLogs}
            <div class="max-h-48 overflow-y-auto font-mono text-[10px] p-3 rounded-xl bg-black/60 border border-zinc-800 text-zinc-400 space-y-0.5 select-text">
              {#each dryRunLogs as line}
                <div class="{line.includes('err:') || line.includes('Error:') ? 'text-rose-400' : line.includes('warn:') ? 'text-amber-400' : 'text-zinc-400'}">
                  {line}
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Modal Actions -->
        <div class="flex justify-end pt-1">
          <button
            onclick={() => (isDryRunDrawerOpen = false)}
            class="px-5 py-2.5 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-semibold active:scale-95 transition-all shadow-sm cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Center Bottom Version Footer -->
  <footer class="py-2.5 text-center text-[11px] text-zinc-500 font-mono select-none shrink-0 border-t border-white/[0.03]">
    Swifter v0.1 • by msandypr
  </footer>
</div>
