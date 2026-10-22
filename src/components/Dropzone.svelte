<script lang="ts">
  import { Upload, AlertCircle } from 'lucide-svelte';

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
    class="w-full relative rounded-2xl transition-all duration-200 border-2 border-dashed p-12 flex flex-col items-center justify-center cursor-pointer
      {isDragging
        ? 'border-zinc-300 bg-white/5 scale-[1.01]'
        : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/40 shadow-xl'}"
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
  >
    <div class="flex flex-col items-center text-center space-y-4">
      <div class="w-14 h-14 rounded-2xl bg-zinc-800 border border-white/10 flex items-center justify-center text-zinc-300 shadow-md">
        {#if isAnalyzing}
          <div class="w-6 h-6 border-2 border-zinc-200 border-t-transparent rounded-full animate-spin"></div>
        {:else}
          <Upload class="w-6 h-6 text-zinc-300" />
        {/if}
      </div>

      <div class="space-y-1.5">
        <h2 class="text-lg font-semibold text-zinc-100">
          {isAnalyzing ? 'Inspecting executable...' : 'Drag and drop your Windows executable (.exe) or game folder'}
        </h2>
        <p class="text-xs text-zinc-400 max-w-md leading-relaxed">
          Drop an .exe file or its parent game directory to inspect and package into a native macOS app.
        </p>
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
        class="flex-1 px-4 py-2.5 rounded-xl glass-input text-xs text-zinc-200 placeholder:text-zinc-600 transition-all"
        disabled={isAnalyzing}
      />
      <button
        type="submit"
        disabled={isAnalyzing}
        class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 disabled:opacity-50 text-xs font-medium text-zinc-200 border border-white/10 transition-all shadow-sm"
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
