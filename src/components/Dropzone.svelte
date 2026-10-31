<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { pickFileOrFolder } from '$lib/tauri-bridge';
  import { Upload } from 'lucide-svelte';

  let { onSelectPath, isAnalyzing = false }: {
    onSelectPath: (path: string) => void;
    isAnalyzing?: boolean;
  } = $props();

  let isDragging = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;

    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === 'enter' || event.payload.type === 'over') {
          isDragging = true;
        } else if (event.payload.type === 'drop') {
          isDragging = false;
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            onSelectPath(paths[0]);
          }
        } else if (event.payload.type === 'leave') {
          isDragging = false;
        }
      })
      .then((unsub) => {
        unlisten = unsub;
      })
      .catch((err) => {
        console.warn('Native drag-drop listener failed to register:', err);
      });

    return () => {
      if (unlisten) unlisten();
    };
  });

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

    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0] as unknown as { path?: string; name: string };
      if (file.path) {
        onSelectPath(file.path);
      }
    }
  }

  async function handleClick() {
    if (isAnalyzing) return;
    try {
      const selected = await pickFileOrFolder();
      if (selected) {
        onSelectPath(selected);
      }
    } catch (err) {
      console.warn('Failed to open file picker:', err);
    }
  }
</script>

<div class="w-full">
  <button
    type="button"
    onclick={handleClick}
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
    disabled={isAnalyzing}
    class="w-full relative rounded-2xl transition-all duration-200 p-9 flex flex-col items-center justify-center cursor-pointer border glass-card text-left active:scale-[0.99]
      {isDragging
        ? 'border-zinc-500 bg-white/10 scale-[1.01] shadow-2xl'
        : 'border-zinc-800 hover:border-zinc-700 bg-zinc-900/40 hover:bg-zinc-900/60 shadow-xl'}"
  >
    <div class="flex flex-col items-center text-center space-y-3.5">
      <div class="w-12 h-12 rounded-2xl bg-zinc-800/90 border border-zinc-700/60 flex items-center justify-center text-zinc-300 shadow-sm">
        {#if isAnalyzing}
          <div class="w-5 h-5 border-2 border-zinc-200 border-t-transparent rounded-full animate-spin"></div>
        {:else}
          <Upload class="w-5 h-5 text-zinc-300" />
        {/if}
      </div>

      <div class="space-y-1">
        <h2 class="text-sm font-semibold text-zinc-100">
          {isAnalyzing ? 'Inspecting executable context...' : 'Choose or drop Windows executable (.exe) or application folder'}
        </h2>
        <p class="text-xs text-zinc-400 max-w-sm leading-relaxed">
          Click to browse Finder, or drag & drop files here to convert into a native Mac app.
        </p>
      </div>
    </div>
  </button>
</div>
