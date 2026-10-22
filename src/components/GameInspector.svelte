<script lang="ts">
  import type { SwifterInspection } from '$lib/tauri-bridge';
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
    Play
  } from 'lucide-svelte';

  let {
    inspection,
    onPack,
    onTestRun,
    onBack,
    isPacking = false,
    isDryRunning = false
  }: {
    inspection: SwifterInspection;
    onPack: (options: {
      appName: string;
      backend: string;
      enableMsync: boolean;
      copyFiles: boolean;
    }) => void;
    onTestRun: (exePath: string, backend: string) => void;
    onBack: () => void;
    isPacking?: boolean;
    isDryRunning?: boolean;
  } = $props();

  let appName = $state('');
  let selectedBackend = $state('DXVK');
  let enableMsync = $state(true);
  let copyFiles = $state(false);

  $effect(() => {
    appName = inspection.app_display_name;
    selectedBackend = inspection.recommended_backend;
  });

  // Engine color themes
  const engineThemes: Record<string, { bg: string; text: string; border: string }> = {
    Unity: { bg: 'bg-cyan-500/10', text: 'text-cyan-400', border: 'border-cyan-500/20' },
    Godot: { bg: 'bg-indigo-500/10', text: 'text-indigo-400', border: 'border-indigo-500/20' },
    GameMaker: { bg: 'bg-emerald-500/10', text: 'text-emerald-400', border: 'border-emerald-500/20' },
    Unreal: { bg: 'bg-orange-500/10', text: 'text-orange-400', border: 'border-orange-500/20' },
    Generic: { bg: 'bg-slate-500/10', text: 'text-slate-400', border: 'border-slate-500/20' },
  };

  const currentTheme = $derived(
    engineThemes[inspection.detected_engine] || engineThemes.Generic
  );

  function handleStartPackaging() {
    onPack({
      appName: appName.trim() || inspection.app_display_name,
      backend: selectedBackend,
      enableMsync,
      copyFiles,
    });
  }
</script>

<div class="w-full max-w-3xl mx-auto flex flex-col p-6 space-y-6 overflow-y-auto max-h-[calc(100vh-4rem)]">
  <!-- Top Navigation & Title Bar -->
  <div class="flex items-center justify-between">
    <button
      onclick={onBack}
      disabled={isPacking}
      class="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-white/5 transition-all"
    >
      <ArrowLeft class="w-4 h-4" />
      Back to Dropzone
    </button>

    <div class="flex items-center gap-2">
      <!-- Engine Badge -->
      <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border {currentTheme.bg} {currentTheme.text} {currentTheme.border}">
        <Gamepad2 class="w-3.5 h-3.5" />
        {inspection.detected_engine} Engine
      </span>

      <!-- Architecture Badge -->
      <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-white/5 border border-white/10 text-slate-300">
        <Cpu class="w-3.5 h-3.5 text-sky-400" />
        {inspection.is_64bit ? '64-bit (PE32+)' : '32-bit (PE32)'}
      </span>
    </div>
  </div>

  <!-- Anti-Cheat Warning Banner -->
  {#if inspection.is_anticheat_blocked}
    <div class="rounded-2xl p-4 bg-rose-500/10 border border-rose-500/30 flex items-start gap-3 text-rose-200 text-sm">
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
  <div class="glass-card rounded-2xl p-6 flex flex-col md:flex-row items-center md:items-start gap-6">
    <!-- Icon Preview Container -->
    <div class="w-24 h-24 rounded-2xl bg-slate-900 border border-white/15 p-2 flex items-center justify-center shrink-0 shadow-lg relative group">
      {#if inspection.icon_cached_path}
        <div class="w-full h-full flex flex-col items-center justify-center text-center">
          <Gamepad2 class="w-10 h-10 text-sky-400" />
          <span class="text-[10px] text-slate-400 font-mono mt-1">.icns ready</span>
        </div>
      {:else}
        <Gamepad2 class="w-10 h-10 text-slate-500" />
      {/if}
      <div class="absolute -bottom-1 -right-1 w-6 h-6 rounded-full bg-emerald-500 flex items-center justify-center shadow">
        <Check class="w-3.5 h-3.5 text-white" />
      </div>
    </div>

    <!-- App Details Form -->
    <div class="flex-1 w-full space-y-4">
      <div>
        <label for="app-name-input" class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
          macOS Application Name
        </label>
        <input
          id="app-name-input"
          type="text"
          bind:value={appName}
          class="w-full px-4 py-2 rounded-xl glass-input text-base font-medium"
          placeholder="e.g. My Game"
        />
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs text-slate-400">
        <div class="space-y-1">
          <span class="font-semibold text-slate-500">Target Executable:</span>
          <div class="font-mono text-slate-300 truncate" title={inspection.target_exe_name}>
            {inspection.target_exe_name}
          </div>
        </div>
        <div class="space-y-1">
          <span class="font-semibold text-slate-500">Game Root:</span>
          <div class="font-mono text-slate-300 truncate" title={inspection.game_folder_path}>
            {inspection.game_folder_path}
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Translation Engine Selector -->
  <div class="glass-card rounded-2xl p-6 space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-sm font-semibold text-slate-100 flex items-center gap-2">
          <Layers class="w-4 h-4 text-sky-400" />
          Translation Engine Backend
        </h3>
        <p class="text-xs text-slate-400 mt-0.5">
          Select graphics translation layer based on game graphics API
        </p>
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
      <!-- DXVK Option -->
      <label class="cursor-pointer relative flex flex-col p-4 rounded-xl border transition-all {selectedBackend === 'DXVK' ? 'border-sky-400 bg-sky-500/10' : 'border-white/10 hover:border-white/20 bg-white/5'}">
        <input type="radio" name="backend" value="DXVK" bind:group={selectedBackend} class="sr-only" />
        <span class="font-semibold text-sm text-slate-200">DXVK</span>
        <span class="text-xs text-slate-400 mt-1">DirectX 9–11 via MoltenVK. Best for Unity and indie 3D games.</span>
        {#if inspection.recommended_backend === 'DXVK'}
          <span class="absolute top-2 right-2 px-1.5 py-0.5 text-[9px] rounded font-bold uppercase bg-sky-500/20 text-sky-300 border border-sky-400/30">Recommended</span>
        {/if}
      </label>

      <!-- D3DMetal Option -->
      <label class="cursor-pointer relative flex flex-col p-4 rounded-xl border transition-all {selectedBackend === 'D3DMetal' ? 'border-sky-400 bg-sky-500/10' : 'border-white/10 hover:border-white/20 bg-white/5'}">
        <input type="radio" name="backend" value="D3DMetal" bind:group={selectedBackend} class="sr-only" />
        <span class="font-semibold text-sm text-slate-200">Apple D3DMetal</span>
        <span class="text-xs text-slate-400 mt-1">Game Porting Toolkit 2. Optimized for DirectX 11 & 12.</span>
        {#if inspection.recommended_backend === 'D3DMetal'}
          <span class="absolute top-2 right-2 px-1.5 py-0.5 text-[9px] rounded font-bold uppercase bg-sky-500/20 text-sky-300 border border-sky-400/30">Recommended</span>
        {/if}
      </label>

      <!-- Vanilla Wine Option -->
      <label class="cursor-pointer relative flex flex-col p-4 rounded-xl border transition-all {selectedBackend === 'Vanilla' ? 'border-sky-400 bg-sky-500/10' : 'border-white/10 hover:border-white/20 bg-white/5'}">
        <input type="radio" name="backend" value="Vanilla" bind:group={selectedBackend} class="sr-only" />
        <span class="font-semibold text-sm text-slate-200">Vanilla Wine</span>
        <span class="text-xs text-slate-400 mt-1">Standard built-in Wine engine. Ideal for 2D GameMaker games.</span>
        {#if inspection.recommended_backend === 'Vanilla'}
          <span class="absolute top-2 right-2 px-1.5 py-0.5 text-[9px] rounded font-bold uppercase bg-sky-500/20 text-sky-300 border border-sky-400/30">Recommended</span>
        {/if}
      </label>
    </div>
  </div>

  <!-- Advanced Options & Storage Mode -->
  <div class="glass-card rounded-2xl p-6 space-y-4">
    <h3 class="text-sm font-semibold text-slate-100 flex items-center gap-2">
      <HardDrive class="w-4 h-4 text-emerald-400" />
      Bundle Packaging Options
    </h3>

    <div class="space-y-3">
      <!-- MSYNC Toggle -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-white/5 hover:bg-white/10 transition-colors cursor-pointer border border-white/5">
        <div class="space-y-0.5">
          <div class="text-sm font-medium text-slate-200">Enable MSYNC / ESYNC</div>
          <div class="text-xs text-slate-400">High-performance synchronization primitives for lower frame latency.</div>
        </div>
        <input type="checkbox" bind:checked={enableMsync} class="w-4 h-4 rounded text-sky-500 focus:ring-sky-400 accent-sky-500" />
      </label>

      <!-- Copy vs Symlink Toggle -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-white/5 hover:bg-white/10 transition-colors cursor-pointer border border-white/5">
        <div class="space-y-0.5">
          <div class="text-sm font-medium text-slate-200">Standalone Bundle (Copy all files into .app)</div>
          <div class="text-xs text-slate-400">
            {copyFiles ? 'Creates a completely self-contained .app bundle. Downloaded game folder can be safely deleted.' : 'Uses fast symlink to preserve disk space (~15MB per game micro-prefix).'}
          </div>
        </div>
        <input type="checkbox" bind:checked={copyFiles} class="w-4 h-4 rounded text-sky-500 focus:ring-sky-400 accent-sky-500" />
      </label>
    </div>
  </div>

  <!-- Action Buttons -->
  <div class="flex items-center justify-end gap-3 pt-2">
    <!-- 3-Second Dry-Run Button -->
    <button
      onclick={() => onTestRun(inspection.target_exe_path, selectedBackend)}
      disabled={isDryRunning || isPacking || inspection.is_anticheat_blocked}
      class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border border-white/15 bg-white/5 hover:bg-white/10 active:scale-95 disabled:opacity-50 text-sm font-medium text-slate-300 transition-all"
    >
      {#if isDryRunning}
        <div class="w-4 h-4 border-2 border-slate-300 border-t-transparent rounded-full animate-spin"></div>
        <span>Testing...</span>
      {:else}
        <Terminal class="w-4 h-4 text-sky-400" />
        <span>Dry Run (3s Test)</span>
      {/if}
    </button>

    <!-- Pack Primary Button -->
    <button
      onclick={handleStartPackaging}
      disabled={isPacking || inspection.is_anticheat_blocked}
      class="inline-flex items-center gap-2 px-6 py-2.5 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 active:scale-95 disabled:opacity-50 text-sm font-semibold text-white shadow-lg shadow-sky-600/30 transition-all"
    >
      {#if isPacking}
        <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
        <span>Packaging with Swifter...</span>
      {:else}
        <Package class="w-4 h-4" />
        <span>Pack with Swifter</span>
      {/if}
    </button>
  </div>
</div>
