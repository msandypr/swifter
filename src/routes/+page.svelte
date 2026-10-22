<script lang="ts">
  import {
    inspectTarget,
    packToApp,
    dryRunCheck,
    launchInstalledApp,
    openInFinder,
    type SwifterInspection
  } from '$lib/tauri-bridge';
  import Dropzone from '../components/Dropzone.svelte';
  import GameInspector from '../components/GameInspector.svelte';
  import ProgressModal from '../components/ProgressModal.svelte';
  import Shelf from '../components/Shelf.svelte';
  import { Package, Library, Terminal, X, AlertCircle } from 'lucide-svelte';

  type ActiveTab = 'packer' | 'shelf';

  let activeTab = $state<ActiveTab>('packer');
  let inspection = $state<SwifterInspection | null>(null);
  let isAnalyzing = $state(false);
  let isPacking = $state(false);
  let isDryRunning = $state(false);
  let errorMessage = $state('');

  // Progress modal state
  let isProgressModalOpen = $state(false);
  let isProgressComplete = $state(false);
  let currentProgressStep = $state(1);
  let createdBundlePath = $state('');
  let packedAppName = $state('');

  // Dry-run drawer state
  let isDryRunDrawerOpen = $state(false);
  let dryRunLogs = $state<string[]>([]);

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

<div class="flex-1 flex flex-col h-full overflow-hidden">
  <!-- Segmented Navigation Header -->
  <div class="px-8 pt-4 pb-2 flex items-center justify-between shrink-0">
    <div class="flex items-center p-1 rounded-xl glass-card space-x-1">
      <button
        onclick={() => (activeTab = 'packer')}
        class="inline-flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-medium transition-all {activeTab === 'packer' ? 'bg-sky-500/20 text-sky-300 shadow border border-sky-500/30' : 'text-slate-400 hover:text-slate-200'}"
      >
        <Package class="w-3.5 h-3.5" />
        Packer
      </button>

      <button
        onclick={() => (activeTab = 'shelf')}
        class="inline-flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-medium transition-all {activeTab === 'shelf' ? 'bg-sky-500/20 text-sky-300 shadow border border-sky-500/30' : 'text-slate-400 hover:text-slate-200'}"
      >
        <Library class="w-3.5 h-3.5" />
        Shelf
      </button>
    </div>

    <div class="text-[11px] text-slate-500 font-mono">
      Swifter Engine v2.0
    </div>
  </div>

  <!-- Global Error Banner -->
  {#if errorMessage}
    <div class="mx-8 mt-2 p-3 rounded-xl bg-rose-500/15 border border-rose-500/30 flex items-center justify-between text-rose-200 text-xs">
      <div class="flex items-center gap-2">
        <AlertCircle class="w-4 h-4 text-rose-400 shrink-0" />
        <span>{errorMessage}</span>
      </div>
      <button onclick={() => (errorMessage = '')} class="p-1 hover:bg-rose-500/20 rounded">
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
    {:else}
      <Shelf onNavigatePacker={() => (activeTab = 'packer')} />
    {/if}
  </div>

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
      <div class="w-full max-w-2xl h-[420px] glass-panel rounded-2xl p-5 shadow-2xl flex flex-col space-y-3 border border-white/15">
        <div class="flex items-center justify-between pb-2 border-b border-white/10">
          <div class="flex items-center gap-2 text-xs font-semibold text-slate-200">
            <Terminal class="w-4 h-4 text-sky-400" />
            Wine 3-Second Dry-Run Diagnostic
          </div>
          <button
            onclick={() => (isDryRunDrawerOpen = false)}
            class="p-1 text-slate-400 hover:text-slate-200 rounded-lg hover:bg-white/5 transition-colors"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="flex-1 overflow-y-auto font-mono text-[11px] p-3 rounded-xl bg-black/50 border border-white/5 text-slate-300 space-y-1 select-text">
          {#each dryRunLogs as line}
            <div class="{line.includes('err:') ? 'text-rose-400' : line.includes('warn:') ? 'text-amber-400' : 'text-slate-300'}">
              {line}
            </div>
          {/each}
        </div>

        <div class="flex justify-end pt-1">
          <button
            onclick={() => (isDryRunDrawerOpen = false)}
            class="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/15 text-xs font-medium text-slate-200 transition-colors"
          >
            Close Logs
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
