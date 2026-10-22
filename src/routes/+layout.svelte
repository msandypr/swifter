<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<div class="flex flex-col h-screen w-screen bg-[#0b0e14] text-slate-100 overflow-hidden font-sans select-none">
  <!-- Window Drag Region for macOS Overlay Titlebar -->
  <header
    data-tauri-drag-region
    class="h-9 w-full flex items-center justify-between px-20 border-b border-white/5 bg-slate-950/40 backdrop-blur-xl shrink-0 z-50"
  >
    <div class="text-xs font-medium text-slate-400 tracking-wider flex items-center gap-2 pointer-events-none select-none">
      <span class="inline-block w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
      Swifter
    </div>
  </header>

  <main class="flex-1 overflow-hidden relative flex flex-col">
    {@render children()}
  </main>
</div>
