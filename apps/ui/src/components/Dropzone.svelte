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
    class="w-full relative rounded-xl transition-colors duration-150 p-7 flex flex-col items-center justify-center cursor-pointer border text-left
      {isDragging
        ? 'border-zinc-500 bg-white/[0.06]'
        : 'border-dashed border-white/[0.12] hover:border-white/[0.22] bg-white/[0.02] hover:bg-white/[0.035]'}"
  >
    <div class="flex flex-col items-center text-center space-y-3">
      <div class="w-10 h-10 rounded-xl bg-zinc-800/90 border border-white/[0.08] flex items-center justify-center text-zinc-300">
        {#if isAnalyzing}
          <div class="w-4 h-4 border-2 border-zinc-200 border-t-transparent rounded-full animate-spin"></div>
        {:else}
          <Upload class="w-4 h-4 text-zinc-300" />
        {/if}
      </div>

      <div class="space-y-1">
        <h2 class="text-[13px] font-semibold text-zinc-100">
          {isAnalyzing ? 'Inspecting executable context...' : 'Choose or drop Windows executable (.exe) or application folder'}
        </h2>
        <p class="text-xs text-zinc-400 max-w-md leading-relaxed">
          Click to browse Finder, or drag & drop files here to convert into a native Mac app.
        </p>
      </div>
    </div>
  </button>
</div>
