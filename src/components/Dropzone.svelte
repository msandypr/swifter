<script lang="ts">
  import { FolderArchive, Upload, Sparkles, AlertCircle } from 'lucide-svelte';

  let { onSelectPath, isAnalyzing = false }: {
    onSelectPath: (path: string) => void;
    isAnalyzing?: boolean;
  } = $props();

  let isDragging = $state(false);
  let manualPath = $state('');
  let inputError = $state('');

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    inputError = '';

    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      // In web/tauri, file objects have a path property
      const file = files[0] as unknown as { path?: string; name: string };
      if (file.path) {
        onSelectPath(file.path);
        return;
      }
    }

    inputError = 'Could not resolve local file path. Please type or paste the absolute path below.';
  }

  function handleManualSubmit(e: SubmitEvent) {
    e.preventDefault();
    const trimmed = manualPath.trim();
    if (!trimmed) {
      inputError = 'Please enter an executable (.exe) or game folder path.';
      return;
    }
    inputError = '';
    onSelectPath(trimmed);
  }
</script>

<div class="w-full max-w-2xl mx-auto flex flex-col items-center justify-center p-6 space-y-6">
  <!-- Drag-and-drop Card -->
  <div
    role="region"
    aria-label="File dropzone"
    class="w-full relative group rounded-2xl transition-all duration-300 border-2 border-dashed p-10 flex flex-col items-center justify-center cursor-pointer
      {isDragging
        ? 'border-sky-400 bg-sky-500/10 shadow-[0_0_30px_rgba(56,189,248,0.2)] scale-[1.01]'
        : 'border-white/15 hover:border-white/30 glass-card shadow-xl'}"
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
  >
    <!-- Background subtle glow -->
    <div class="absolute inset-0 bg-gradient-to-b from-sky-500/5 to-transparent rounded-2xl pointer-events-none"></div>

    <div class="relative z-10 flex flex-col items-center text-center space-y-4">
      <div class="w-16 h-16 rounded-2xl bg-gradient-to-tr from-sky-600 to-indigo-600 flex items-center justify-center shadow-lg shadow-sky-600/30 group-hover:scale-105 transition-transform">
        {#if isAnalyzing}
          <div class="w-8 h-8 border-3 border-white border-t-transparent rounded-full animate-spin"></div>
        {:else}
          <Upload class="w-8 h-8 text-white" />
        {/if}
      </div>

      <div class="space-y-1">
        <h2 class="text-xl font-semibold text-slate-100">
          {isAnalyzing ? 'Inspecting binary & dependencies...' : 'Drop Windows .exe or Game Folder'}
        </h2>
        <p class="text-sm text-slate-400 max-w-md">
          Drag DRM-free game folders from itch.io, indie packages, or utility software. Swifter preserves full folder assets.
        </p>
      </div>

      <div class="flex items-center gap-2 pt-2">
        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-white/5 border border-white/10 text-slate-300">
          <FolderArchive class="w-3.5 h-3.5 text-sky-400" />
          Unity, Godot & GameMaker
        </span>
        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-white/5 border border-white/10 text-slate-300">
          <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
          Zero-Config Packaging
        </span>
      </div>
    </div>
  </div>

  <!-- Manual Path Input Fallback -->
  <form onsubmit={handleManualSubmit} class="w-full flex flex-col space-y-2">
    <div class="flex items-center gap-2">
      <input
        type="text"
        bind:value={manualPath}
        placeholder="Or paste full path: /Users/.../Games/MyGame (or MyGame.exe)"
        class="flex-1 px-4 py-2.5 rounded-xl glass-input text-sm placeholder:text-slate-500 transition-all"
        disabled={isAnalyzing}
      />
      <button
        type="submit"
        disabled={isAnalyzing}
        class="px-5 py-2.5 rounded-xl bg-sky-600 hover:bg-sky-500 active:scale-95 disabled:opacity-50 text-sm font-medium text-white transition-all shadow-md shadow-sky-600/25"
      >
        Inspect
      </button>
    </div>

    {#if inputError}
      <div class="flex items-center gap-2 text-rose-400 text-xs px-1">
        <AlertCircle class="w-4 h-4 shrink-0" />
        <span>{inputError}</span>
      </div>
    {/if}
  </form>
</div>
