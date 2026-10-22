<script lang="ts">
  import {
    BookOpen,
    Layers,
    Cpu,
    HardDrive,
    PackageCheck,
    Wrench,
    FileCode
  } from 'lucide-svelte';
</script>

<div class="w-full max-w-3xl mx-auto flex flex-col space-y-6 pb-8 text-zinc-300">
  <!-- Guide Header -->
  <div class="border-b border-zinc-800 pb-4 space-y-1">
    <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
      <BookOpen class="w-4 h-4 text-zinc-400" />
      Swifter User Guide
    </h2>
    <p class="text-xs text-zinc-400">
      Step-by-step instructions for converting Windows games and installers into native macOS .app bundles.
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
        <h3 class="text-sm font-semibold text-zinc-100">Prerequisite: Wine Translation Runtime</h3>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-9">
        Swifter creates the native macOS application wrapper and isolated prefix. To run the game, macOS requires a Wine runtime. You can check the status at any time via the <strong>Requirements</strong> tab in the navigation bar:
      </p>
      <div class="pl-9 space-y-2">
        <div class="p-3 rounded-xl bg-zinc-950 border border-zinc-800 space-y-2 text-xs">
          <div class="text-zinc-200 font-medium">Two easy installation methods:</div>
          <ul class="list-disc list-inside space-y-1 text-[11px] text-zinc-400">
            <li><strong>In-App 1-Click:</strong> Open the <strong>Requirements</strong> tab and click "Install Swifter Runtime". Swifter downloads and unpacks a pre-built portable runtime automatically.</li>
            <li><strong>Via Homebrew:</strong> Run <code class="text-zinc-300 bg-zinc-900 px-1 py-0.5 rounded">brew install --cask wine-stable</code> in your terminal. Swifter detects it immediately.</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- Step 2 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          2
        </div>
        <h3 class="text-sm font-semibold text-zinc-100">Drop Executable or Game Folder</h3>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-9">
        Drag and drop either an executable (<code class="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-[11px]">.exe</code>) or an unpacked game folder. Swifter inspects the directory structure, keeping all companion files (Unity <code class="text-zinc-300 text-[11px]">*_Data</code>, Godot <code class="text-zinc-300 text-[11px]">.pck</code>, GameMaker <code class="text-zinc-300 text-[11px]">data.win</code>, and companion DLLs) intact.
      </p>
    </div>

    <!-- Step 3 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          3
        </div>
        <h3 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
          Handling Installer Wizards
        </h3>
      </div>
      <div class="pl-9 space-y-2 text-xs text-zinc-400">
        <p class="leading-relaxed">
          Some games are distributed as installer wizards rather than standalone portable games:
        </p>
        <div class="p-3 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-1.5">
          <div class="font-semibold text-zinc-200">Recommended Installer Workflow:</div>
          <ol class="list-decimal list-inside space-y-1 text-[11px] text-zinc-400">
            <li>When you drop a setup executable, Swifter highlights <strong>"Setup Wizard Detected"</strong>.</li>
            <li>Click <strong>"Run Setup Wizard in Micro-Prefix"</strong>. Complete the installation wizard in Wine.</li>
            <li>Once the installer finishes, Swifter automatically scans the prefix and selects the newly installed game executable.</li>
            <li>Click <strong>"Pack with Swifter"</strong>. The launcher will now launch the real game instead of re-running the installer!</li>
          </ol>
        </div>
        <div class="p-2.5 rounded-xl bg-zinc-900/60 border border-zinc-800 text-[11px]">
          <strong>Already packaged an installer by mistake?</strong> Go to the <strong>Shelf</strong> tab, click the <code class="text-zinc-200 bg-zinc-800 px-1 py-0.5 rounded">.exe</code> button on the game card, and switch the launch target to the installed game binary.
        </div>
      </div>
    </div>

    <!-- Step 4 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          4
        </div>
        <h3 class="text-sm font-semibold text-zinc-100">Graphics Backends & Translation Layers</h3>
      </div>
      <div class="pl-9 space-y-2 text-xs text-zinc-400">
        <div class="p-2.5 rounded-xl bg-zinc-950/60 border border-zinc-800 space-y-1">
          <span class="font-semibold text-zinc-200">DXVK (Default for 3D Titles):</span>
          <p class="text-[11px] text-zinc-400">Translates DirectX 9, 10, and 11 into Vulkan / MoltenVK on macOS for high-performance rendering.</p>
        </div>
        <div class="p-2.5 rounded-xl bg-zinc-950/60 border border-zinc-800 space-y-1">
          <span class="font-semibold text-zinc-200">Apple D3DMetal (Game Porting Toolkit 2):</span>
          <p class="text-[11px] text-zinc-400">Apple's native DirectX 11/12 to Metal translation engine for heavy graphical workloads.</p>
        </div>
        <div class="p-2.5 rounded-xl bg-zinc-950/60 border border-zinc-800 space-y-1">
          <span class="font-semibold text-zinc-200">Vanilla Wine & OpenGL Zink:</span>
          <p class="text-[11px] text-zinc-400">Used for 2D software and OpenGL games (like Geometry Dash). Automatically routes OpenGL calls over Vulkan/Metal via Mesa Zink.</p>
        </div>
      </div>
    </div>

    <!-- Step 5 -->
    <div class="glass-card rounded-2xl p-5 border border-zinc-800 bg-zinc-900/40 space-y-3">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-semibold text-zinc-200">
          5
        </div>
        <h3 class="text-sm font-semibold text-zinc-100">Isolated Micro-Prefixes & Save Data</h3>
      </div>
      <p class="text-xs text-zinc-400 leading-relaxed pl-9">
        Unlike traditional Wine tools that clone 2 GB virtual C: drives for each game, Swifter creates a micro-prefix (~15 MB) using symlinks to shared base runtime files. Save data and user configurations in <code class="text-zinc-200 bg-zinc-800 px-1 py-0.5 rounded text-[11px]">AppData</code> remain strictly independent per game and can be accessed with one click via the <strong>Save Data</strong> button on the <strong>Shelf</strong>.
      </p>
    </div>
  </div>
</div>
