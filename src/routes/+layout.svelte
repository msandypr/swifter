<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<div class="flex flex-col h-screen w-screen bg-[#111113] text-zinc-100 overflow-hidden font-sans select-none">
  <!-- Window Drag Region for macOS Overlay Titlebar -->
  <header
    data-tauri-drag-region
    class="h-9 w-full flex items-center justify-between px-20 border-b border-white/5 bg-zinc-950/60 backdrop-blur-xl shrink-0 z-50"
  >
    <div class="text-xs font-medium text-zinc-400 tracking-wider pointer-events-none select-none">
      Swifter
    </div>
  </header>

  <main class="flex-1 overflow-hidden relative flex flex-col">
    {@render children()}
  </main>
</div>
