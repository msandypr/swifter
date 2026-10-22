<script lang="ts">
  import { BookOpen, Terminal, Layers, HardDrive, ShieldCheck } from 'lucide-svelte';
</script>

<div class="w-full max-w-3xl mx-auto flex flex-col p-6 space-y-6 overflow-y-auto max-h-[calc(100vh-4rem)] text-zinc-300">
  <!-- Guide Header -->
  <div class="border-b border-zinc-800 pb-4 space-y-1">
    <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
      <BookOpen class="w-4 h-4 text-zinc-400" />
      Swifter User Guide
    </h2>
    <p class="text-xs text-zinc-400">
      Step-by-step instructions for converting Windows games into native macOS .app bundles.
    </p>
  </div>

  <!-- Steps List -->
  <div class="space-y-4">
    <!-- Step 1 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          1
        </div>
        <h3 class="text-sm font-semibold text-zinc-100">Prerequisite: Wine Runtime</h3>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-9">
        Swifter creates the native macOS application wrapper and isolated prefix. To run the game, macOS requires a Wine runtime. Swifter automatically detects any of the following runtimes on your system:
      </p>
      <div class="pl-9 space-y-2">
        <div class="p-3 rounded-xl bg-zinc-950 border border-zinc-800 font-mono text-xs text-zinc-300">
          # Install via Homebrew (recommended):<br/>
          brew install --cask wine-stable
        </div>
        <p class="text-[11px] text-zinc-500">
          Alternatively, if you have Whisky or CrossOver installed, Swifter will automatically detect and link to their bundled wine binaries.
        </p>
      </div>
    </div>

    <!-- Step 2 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          2
        </div>
        <h3 class="text-sm font-semibold text-zinc-100">Drop Game Executable or Folder</h3>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-9">
        Drag and drop either a single <code class="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-[11px]">.exe</code> file or the entire game folder. Swifter automatically inspects the folder context, keeping Unity data folders (<code class="text-zinc-300 text-[11px]">*_Data</code>), Godot packages (<code class="text-zinc-300 text-[11px]">.pck</code>), and DLL libraries intact.
      </p>
    </div>

    <!-- Step 3 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          3
        </div>
        <h3 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
          Choose Translation Backend
        </h3>
      </div>
      <div class="pl-9 space-y-2 text-xs text-zinc-400">
        <div class="p-2.5 rounded-xl bg-zinc-950/60 border border-zinc-800 space-y-1">
          <span class="font-semibold text-zinc-200">DXVK (Default for 3D Indie Games):</span>
          <p class="text-[11px] text-zinc-400">Translates DirectX 9, 10, and 11 calls into Vulkan / MoltenVK on macOS. High frame-rate performance for Unity and Unreal indie games.</p>
        </div>
        <div class="p-2.5 rounded-xl bg-zinc-950/60 border border-zinc-800 space-y-1">
          <span class="font-semibold text-zinc-200">Apple D3DMetal (Game Porting Toolkit 2):</span>
          <p class="text-[11px] text-zinc-400">Apple's native DirectX 11/12 to Metal translation layer for heavier graphical workloads.</p>
        </div>
        <div class="p-2.5 rounded-xl bg-zinc-950/60 border border-zinc-800 space-y-1">
          <span class="font-semibold text-zinc-200">Vanilla Wine:</span>
          <p class="text-[11px] text-zinc-400">Built-in OpenGL/DirectX translation. Best suited for 2D GameMaker games and lightweight utilities.</p>
        </div>
      </div>
    </div>

    <!-- Step 4 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          4
        </div>
        <h3 class="text-sm font-semibold text-zinc-100">Isolated Micro-Prefixes & Save Data</h3>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-9">
        Traditional Wine tools duplicate a 2GB virtual C: drive for every single game. Swifter instead provisions an isolated micro-prefix (~15MB) using symlinks to shared base runtime files. Save files and configuration in <code class="text-zinc-200 bg-zinc-800 px-1 py-0.5 rounded text-[11px]">AppData</code> remain completely independent for each game and can be accessed with one click from the <strong>Shelf</strong> tab.
      </p>
    </div>
  </div>
</div>
