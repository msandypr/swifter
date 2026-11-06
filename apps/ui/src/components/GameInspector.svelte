<script lang="ts">
  import { runInstallerInPrefix, type SwifterInspection } from '$lib/tauri-bridge';
  import {
    Cpu,
    Gamepad2,
    ShieldAlert,
    Package,
    Terminal,
    ArrowLeft,
    Check,
    HardDrive,
    Layers,
    Play,
    AlertTriangle,
    Lock,
    ChevronDown,
    Monitor
  } from 'lucide-svelte';
  let {
    inspection,
    hasD3DMetal = false,
    onPack,
    onTestRun,
    onBack,
    isPacking = false,
    isDryRunning = false
  }: {
    inspection: SwifterInspection;
    hasD3DMetal?: boolean;
    onPack: (options: {
      appName: string;
      targetExe?: string;
      backend: string;
      enableMsync: boolean;
      copyFiles: boolean;
      launchArgs?: string;
      customEnv?: Record<string, string>;
      enableRetina?: boolean;
      dpi?: number;
    }) => void;
    onTestRun: (exePath: string, backend: string, launchArgs?: string, customEnv?: Record<string, string>) => void;
    onBack: () => void;
    isPacking?: boolean;
    isDryRunning?: boolean;
  } = $props();

  let appName = $state('');
  let selectedBackend = $state('DXVK');
  let enableMsync = $state(true);
  let copyFiles = $state(false);
  let enableRetina = $state(true);
  let dpi = $state(144);

  // Launch arguments & custom environment variables
  let launchArgs = $state('');
  let customEnvText = $state('');
  let showEnvFields = $state(false);

  function parseEnvLines(text: string): Record<string, string> {
    const result: Record<string, string> = {};
    for (const line of text.split('\n')) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#')) continue;
      const eqIdx = trimmed.indexOf('=');
      if (eqIdx > 0) {
        const k = trimmed.slice(0, eqIdx).trim();
        const v = trimmed.slice(eqIdx + 1).trim();
        if (k) result[k] = v;
      }
    }
    return result;
  }

  // Target executable and installer states
  let selectedTargetExe = $state('');
  let isRunningInstaller = $state(false);
  let installerDiscoveredExes = $state<string[]>([]);
  let installerMessage = $state('');

  $effect(() => {
    appName = inspection.app_display_name;
    selectedBackend = inspection.recommended_backend;
    selectedTargetExe = inspection.target_exe_name;
  });

  // Combine original candidates and post-installer discoveries
  const allExeOptions = $derived(() => {
    const set = new Set<string>();
    if (selectedTargetExe) set.add(selectedTargetExe);
    set.add(inspection.target_exe_name);
    for (const c of inspection.exe_candidates) {
      set.add(c);
    }
    for (const d of installerDiscoveredExes) {
      set.add(d);
    }
    return Array.from(set);
  });

  async function handleRunInstaller() {
    isRunningInstaller = true;
    installerMessage = 'Installer wizard running in Wine... Please complete the installation steps.';
    try {
      const slug = (appName.trim() || inspection.app_display_name).toLowerCase().replace(/[^a-z0-9]+/g, '-');
      const discovered = await runInstallerInPrefix(inspection.target_exe_path, slug);
      installerDiscoveredExes = discovered;
      if (discovered.length > 0) {
        selectedTargetExe = discovered[0];
        installerMessage = `Found ${discovered.length} installed game binary(ies)! Selected: ${discovered[0]}`;
      } else {
        installerMessage = 'Installer completed. You can verify the executable target below.';
      }
    } catch (err: unknown) {
      installerMessage = err instanceof Error ? err.message : String(err);
    } finally {
      isRunningInstaller = false;
    }
  }

  function handleStartPackaging() {
    const envObj = parseEnvLines(customEnvText);
    onPack({
      appName: appName.trim() || inspection.app_display_name,
      targetExe: selectedTargetExe || inspection.target_exe_name,
      backend: selectedBackend,
      enableMsync,
      copyFiles,
      launchArgs: launchArgs.trim() || undefined,
      customEnv: Object.keys(envObj).length > 0 ? envObj : undefined,
      enableRetina,
      dpi,
    });
  }
</script>

<div class="w-full max-w-3xl mx-auto flex flex-col p-6 space-y-6 overflow-y-auto max-h-[calc(100vh-4rem)]">
  <!-- Top Navigation & Title Bar -->
  <div class="flex items-center justify-between">
    <button
      onclick={onBack}
      disabled={isPacking}
      class="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-zinc-400 hover:text-zinc-200 hover:bg-white/5 transition-all"
    >
      <ArrowLeft class="w-4 h-4" />
      Back to Dropzone
    </button>

    <div class="flex items-center gap-2">
      <!-- Engine Badge -->
      <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-zinc-800 text-zinc-300 border border-zinc-700">
        <Gamepad2 class="w-3.5 h-3.5 text-zinc-400" />
        {inspection.detected_engine} Engine
      </span>

      <!-- Architecture Badge -->
      <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-zinc-800 text-zinc-300 border border-zinc-700">
        <Cpu class="w-3.5 h-3.5 text-zinc-400" />
        {inspection.is_64bit ? '64-bit (PE32+)' : '32-bit (PE32)'}
      </span>
    </div>
  </div>

  <!-- Installer / Setup Wizard Banner -->
  {#if inspection.is_installer}
    <div class="rounded-2xl p-4 bg-zinc-900 border border-amber-500/30 flex flex-col space-y-3">
      <div class="flex items-start gap-3 text-amber-200 text-xs">
        <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
        <div class="space-y-1">
          <div class="font-semibold text-amber-300">Setup Wizard / Installer Executable Detected</div>
          <p class="text-zinc-400 text-[11px] leading-relaxed">
            This executable is an installer. To package the actual application, run the setup wizard first so it installs into the isolated prefix, or select the installed application executable below.
          </p>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2 pt-1">
        <button
          onclick={handleRunInstaller}
          disabled={isRunningInstaller}
          class="inline-flex items-center gap-1.5 px-4 py-2 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 font-semibold text-xs active:scale-95 transition-all shadow-sm disabled:opacity-50"
        >
          {#if isRunningInstaller}
            <div class="w-3.5 h-3.5 border-2 border-zinc-900 border-t-transparent rounded-full animate-spin"></div>
            <span>Running Installer...</span>
          {:else}
            <Play class="w-3.5 h-3.5" />
            <span>Run Setup Wizard in Micro-Prefix</span>
          {/if}
        </button>

        {#if installerMessage}
          <span class="text-[11px] text-zinc-300 font-mono px-2 py-1 rounded bg-zinc-950 border border-zinc-800">
            {installerMessage}
          </span>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Anti-Cheat Warning Banner -->
  {#if inspection.is_anticheat_blocked}
    <div class="rounded-2xl p-4 bg-zinc-900 border border-rose-500/40 flex items-start gap-3 text-rose-200 text-sm">
      <ShieldAlert class="w-5 h-5 text-rose-400 shrink-0 mt-0.5" />
      <div class="space-y-1">
        <div class="font-semibold text-rose-300">Kernel Anti-Cheat Detected</div>
        <p class="text-xs text-rose-200/80 leading-relaxed">
          {inspection.anticheat_reason || 'This game contains a Windows kernel-level anti-cheat driver which cannot run under Wine on macOS.'}
        </p>
      </div>
    </div>
  {/if}

  <!-- Main Game Card -->
  <div class="glass-card rounded-2xl p-6 flex flex-col md:flex-row items-center md:items-start gap-6 border border-zinc-800 bg-zinc-900/40">
    <!-- Icon Preview Container -->
    <div class="w-24 h-24 rounded-2xl bg-zinc-800 border border-white/10 p-2 flex items-center justify-center shrink-0 shadow-lg relative">
      {#if inspection.icon_cached_path}
        <div class="w-full h-full flex flex-col items-center justify-center text-center">
          <Gamepad2 class="w-10 h-10 text-zinc-300" />
          <span class="text-[10px] text-zinc-400 font-mono mt-1">.icns</span>
        </div>
      {:else}
        <Gamepad2 class="w-10 h-10 text-zinc-500" />
      {/if}
      <div class="absolute -bottom-1 -right-1 w-6 h-6 rounded-full bg-zinc-200 flex items-center justify-center shadow">
        <Check class="w-3.5 h-3.5 text-zinc-900" />
      </div>
    </div>

    <!-- App Details Form -->
    <div class="flex-1 w-full space-y-4">
      <div>
        <label for="app-name-input" class="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1.5">
          macOS Application Name
        </label>
        <input
          id="app-name-input"
          type="text"
          bind:value={appName}
          class="w-full px-4 py-2 rounded-xl glass-input text-sm font-medium border border-zinc-800 bg-zinc-950/60 text-zinc-100"
          placeholder="e.g. My Application"
        />
      </div>

      <!-- Target Executable Selector / Dropdown -->
      <div>
        <label for="target-exe-select" class="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1.5">
          Executable to Launch
        </label>
        {#if allExeOptions().length > 1}
          <div class="relative">
            <select
              id="target-exe-select"
              bind:value={selectedTargetExe}
              class="w-full h-11 pl-4 pr-10 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 hover:border-zinc-700 focus:border-zinc-600 bg-zinc-950/80 cursor-pointer appearance-none transition-colors"
            >
              {#each allExeOptions() as opt}
                <option value={opt} class="bg-zinc-900 text-zinc-200 py-1">{opt}</option>
              {/each}
            </select>
            <div class="pointer-events-none absolute right-3.5 top-1/2 -translate-y-1/2 text-zinc-400">
              <ChevronDown class="w-4 h-4" />
            </div>
          </div>
        {:else}
          <input
            id="target-exe-select"
            type="text"
            bind:value={selectedTargetExe}
            class="w-full h-11 px-4 rounded-xl glass-input text-xs font-mono text-zinc-200 border border-zinc-800 bg-zinc-950/80"
          />
        {/if}
      </div>

      <div class="text-xs text-zinc-400 space-y-1">
        <span class="font-semibold text-zinc-500">Root Folder:</span>
        <div class="font-mono text-zinc-300 truncate text-[11px]" title={inspection.game_folder_path}>
          {inspection.game_folder_path}
        </div>
      </div>
    </div>
  </div>

  <!-- Translation Engine Selector -->
  <div class="glass-card rounded-2xl p-6 space-y-4 border border-zinc-800 bg-zinc-900/40">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
          <Layers class="w-4 h-4 text-zinc-400" />
          Translation Engine Backend
        </h3>
        <p class="text-xs text-zinc-400 mt-0.5">
          Select graphics translation layer based on application graphics API
        </p>
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
      <!-- DXVK Option -->
      <label class="cursor-pointer relative flex flex-col p-4 rounded-xl border transition-all {selectedBackend === 'DXVK' ? 'border-zinc-300 bg-zinc-800 text-zinc-100' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 text-zinc-400'}">
        <input type="radio" name="backend" value="DXVK" bind:group={selectedBackend} class="sr-only" />
        <span class="font-semibold text-sm text-zinc-200">DXVK</span>
        <span class="text-xs text-zinc-400 mt-1">DirectX 9–11 via MoltenVK. Best for 3D indie titles.</span>
        {#if inspection.recommended_backend === 'DXVK'}
          <span class="absolute top-2.5 right-2.5 px-1.5 py-0.5 text-[9px] rounded font-bold uppercase bg-zinc-700 text-zinc-200 border border-zinc-600">Recommended</span>
        {/if}
      </label>

      <!-- D3DMetal Option -->
      <label class="cursor-pointer relative flex flex-col p-4 rounded-xl border transition-all {selectedBackend === 'D3DMetal' ? 'border-zinc-300 bg-zinc-800 text-zinc-100' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 text-zinc-400'}">
        <input type="radio" name="backend" value="D3DMetal" bind:group={selectedBackend} class="sr-only" />
        <div class="flex items-center justify-between pr-2">
          <span class="font-semibold text-sm text-zinc-200">Apple D3DMetal</span>
          {#if !hasD3DMetal}
            <span class="px-1.5 py-0.5 text-[9px] rounded font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20">Requires GPTK</span>
          {/if}
        </div>
        <span class="text-xs text-zinc-400 mt-1">Game Porting Toolkit 2. Optimized for DirectX 11 & 12.</span>
        {#if inspection.recommended_backend === 'D3DMetal'}
          <span class="absolute top-2.5 right-2.5 px-1.5 py-0.5 text-[9px] rounded font-bold uppercase bg-zinc-700 text-zinc-200 border border-zinc-600">Recommended</span>
        {/if}
      </label>

      <!-- Vanilla Wine Option -->
      <label class="cursor-pointer relative flex flex-col p-4 rounded-xl border transition-all {selectedBackend === 'Vanilla' ? 'border-zinc-300 bg-zinc-800 text-zinc-100' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 text-zinc-400'}">
        <input type="radio" name="backend" value="Vanilla" bind:group={selectedBackend} class="sr-only" />
        <span class="font-semibold text-sm text-zinc-200">Vanilla Wine & OpenGL</span>
        <span class="text-xs text-zinc-400 mt-1">Built-in Wine + WineMetalGL (OpenGL 1.0–4.6 via Metal).</span>
        {#if inspection.recommended_backend === 'Vanilla'}
          <span class="absolute top-2.5 right-2.5 px-1.5 py-0.5 text-[9px] rounded font-bold uppercase bg-zinc-700 text-zinc-200 border border-zinc-600">Recommended</span>
        {/if}
      </label>
    </div>

    {#if selectedBackend === 'D3DMetal' && !hasD3DMetal}
      <div class="p-3 rounded-xl bg-amber-950/30 border border-amber-500/30 text-xs text-amber-300 flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 text-amber-400 mt-0.5" />
        <span>
          <strong>Apple D3DMetal not detected</strong> in the current Wine runtime. Unless using Whisky or an Apple Game Porting Toolkit build, titles requiring D3D11/12 may fail to launch. <strong>DXVK</strong> is recommended for standard setups.
        </span>
      </div>
    {/if}
  </div>

  <!-- Advanced Options & Storage Mode -->
  <div class="glass-card rounded-2xl p-6 space-y-4 border border-zinc-800 bg-zinc-900/40">
    <h3 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
      <HardDrive class="w-4 h-4 text-zinc-400" />
      Bundle Packaging Options
    </h3>

    <div class="space-y-3">
      <!-- MSYNC Toggle -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-zinc-800/40 hover:bg-zinc-800/60 transition-colors cursor-pointer border border-zinc-800">
        <div class="space-y-0.5">
          <div class="text-sm font-medium text-zinc-200">Enable MSYNC / ESYNC</div>
          <div class="text-xs text-zinc-400">High-performance synchronization primitives for lower frame latency.</div>
        </div>
        <input type="checkbox" bind:checked={enableMsync} class="w-4 h-4 rounded accent-zinc-400" />
      </label>

      <!-- Copy vs Symlink Toggle -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-zinc-800/40 hover:bg-zinc-800/60 transition-colors cursor-pointer border border-zinc-800">
        <div class="space-y-0.5">
          <div class="text-sm font-medium text-zinc-200">Standalone Bundle (Copy all files into .app)</div>
          <div class="text-xs text-zinc-400">
            {copyFiles ? 'Creates a completely self-contained .app bundle. Original source folder can be safely deleted.' : 'Uses fast symlink to preserve disk space (micro-prefix).'}
          </div>
        </div>
        <input type="checkbox" bind:checked={copyFiles} class="w-4 h-4 rounded accent-zinc-400" />
      </label>
    </div>
  </div>

  <!-- Display & High-DPI Scaling -->
  <div class="glass-card rounded-2xl p-6 space-y-4 border border-zinc-800 bg-zinc-900/40">
    <div>
      <h3 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
        <Monitor class="w-4 h-4 text-zinc-400" />
        Display & High-DPI Scaling
      </h3>
      <p class="text-xs text-zinc-400 mt-0.5">
        Configure Retina buffer rendering and Windows DPI scaling presets
      </p>
    </div>

    <!-- Retina Mode Switch -->
    <label class="flex items-center justify-between p-3 rounded-xl bg-zinc-800/40 hover:bg-zinc-800/60 transition-colors cursor-pointer border border-zinc-800">
      <div class="space-y-0.5 pr-4">
        <div class="text-sm font-medium text-zinc-200">High-DPI Retina Mode</div>
        <div class="text-xs text-zinc-400">
          Enables crisp native 2x Retina rendering instead of macOS blurry non-integer upscaling.
        </div>
      </div>
      <input type="checkbox" bind:checked={enableRetina} class="w-4 h-4 rounded accent-zinc-400 shrink-0" />
    </label>

    <!-- DPI Presets -->
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Windows UI Scale (DPI)</span>
        <span class="text-xs font-mono text-zinc-400">
          {dpi === 96 ? '100% (Standard 96 DPI)' : dpi === 120 ? '125% (Medium 120 DPI)' : dpi === 144 ? '150% (Recommended 144 DPI)' : dpi === 192 ? '200% (Large 192 DPI)' : `${dpi} DPI`}
        </span>
      </div>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
        <button
          type="button"
          onclick={() => (dpi = 96)}
          class="py-2.5 px-3 rounded-xl border text-center transition-all cursor-pointer {dpi === 96 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 text-zinc-400'}"
        >
          <div class="font-semibold text-xs">100%</div>
          <div class="text-[10px] text-zinc-500 font-mono">96 DPI (Default)</div>
        </button>

        <button
          type="button"
          onclick={() => (dpi = 120)}
          class="py-2.5 px-3 rounded-xl border text-center transition-all cursor-pointer {dpi === 120 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 text-zinc-400'}"
        >
          <div class="font-semibold text-xs">125%</div>
          <div class="text-[10px] text-zinc-500 font-mono">120 DPI</div>
        </button>

        <button
          type="button"
          onclick={() => (dpi = 144)}
          class="py-2.5 px-3 rounded-xl border text-center transition-all cursor-pointer {dpi === 144 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 text-zinc-400'}"
        >
          <div class="font-semibold text-xs">150%</div>
          <div class="text-[10px] text-zinc-500 font-mono">144 DPI (Retina)</div>
        </button>

        <button
          type="button"
          onclick={() => (dpi = 192)}
          class="py-2.5 px-3 rounded-xl border text-center transition-all cursor-pointer {dpi === 192 ? 'border-zinc-300 bg-zinc-800 text-zinc-100 shadow-sm' : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 text-zinc-400'}"
        >
          <div class="font-semibold text-xs">200%</div>
          <div class="text-[10px] text-zinc-500 font-mono">192 DPI (Large)</div>
        </button>
      </div>
    </div>
  </div>

  <!-- Launch Arguments & Custom Environment Variables -->
  <div class="glass-card rounded-2xl p-6 space-y-4 border border-zinc-800 bg-zinc-900/40">
    <div>
      <h3 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
        <Terminal class="w-4 h-4 text-zinc-400" />
        Launch Arguments & Environment Variables
      </h3>
      <p class="text-xs text-zinc-400 mt-0.5">
        Optional startup flags and environment overrides passed to the application
      </p>
    </div>

    <div class="space-y-3">
      <div>
        <label for="launch-args-input" class="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1.5">
          Command Line Arguments
        </label>
        <input
          id="launch-args-input"
          type="text"
          bind:value={launchArgs}
          class="w-full px-4 py-2 rounded-xl glass-input text-xs font-mono border border-zinc-800 bg-zinc-950/60 text-zinc-100"
          placeholder="e.g. -novid -windowed -force-d3d11"
        />
        <p class="text-[11px] text-zinc-500 mt-1">
          Flags passed directly to the binary at launch.
        </p>
      </div>

      <div>
        <div class="flex items-center justify-between mb-1.5">
          <label for="custom-env-input" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
            Custom Environment Variables
          </label>
          <button
            type="button"
            onclick={() => (showEnvFields = !showEnvFields)}
            class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
          >
            {showEnvFields ? 'Hide' : 'Configure Env'}
          </button>
        </div>

        {#if showEnvFields}
          <textarea
            id="custom-env-input"
            bind:value={customEnvText}
            rows={3}
            class="w-full p-3 rounded-xl glass-input text-xs font-mono border border-zinc-800 bg-zinc-950/60 text-zinc-100 resize-y"
            placeholder="DXVK_HUD=fps&#10;MVK_CONFIG_LOG_LEVEL=1"
          ></textarea>
          <p class="text-[11px] text-zinc-500 mt-1">
            Format: KEY=VALUE (one per line). Useful for DXVK HUD or graphics debugging.
          </p>
        {/if}
      </div>
    </div>
  </div>

  <!-- Action Buttons -->
  <div class="flex items-center justify-end gap-3 pt-2">
    <!-- 3-Second Dry-Run Button -->
    <button
      onclick={() => onTestRun(inspection.target_exe_path, selectedBackend, launchArgs.trim() || undefined, parseEnvLines(customEnvText))}
      disabled={isDryRunning || isPacking || inspection.is_anticheat_blocked || (inspection.is_installer && installerDiscoveredExes.length === 0)}
      class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed text-xs font-medium text-zinc-200 transition-all cursor-pointer"
    >
      {#if isDryRunning}
        <div class="w-3.5 h-3.5 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
        <span>Testing...</span>
      {:else if inspection.is_installer && installerDiscoveredExes.length === 0}
        <Lock class="w-3.5 h-3.5 text-zinc-500" />
        <span>Install First</span>
      {:else}
        <Terminal class="w-3.5 h-3.5 text-zinc-400" />
        <span>Dry Run (3s Test)</span>
      {/if}
    </button>

    <!-- Pack Primary Button -->
    <button
      onclick={handleStartPackaging}
      disabled={isPacking || inspection.is_anticheat_blocked || (inspection.is_installer && installerDiscoveredExes.length === 0)}
      class="inline-flex items-center gap-2 px-6 py-2.5 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed text-xs font-semibold text-zinc-950 border border-zinc-200 transition-all shadow-md cursor-pointer"
    >
      {#if isPacking}
        <div class="w-3.5 h-3.5 border-2 border-zinc-900 border-t-transparent rounded-full animate-spin"></div>
        <span>Packaging with Swifter...</span>
      {:else if inspection.is_installer && installerDiscoveredExes.length === 0}
        <Lock class="w-3.5 h-3.5 text-zinc-700" />
        <span>Run Setup Wizard First</span>
      {:else}
        <Package class="w-3.5 h-3.5" />
        <span>Pack with Swifter</span>
      {/if}
    </button>
  </div>
</div>
