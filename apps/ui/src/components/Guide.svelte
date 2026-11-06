<script lang="ts">
  import { BookOpen, HelpCircle } from 'lucide-svelte';

  const steps = [
    {
      num: '1',
      title: 'Install Wine Runtime',
      desc: 'Open Settings and ensure Wine is installed. Click 1-Click Install if missing.',
      badge: 'Prerequisite'
    },
    {
      num: '2',
      title: 'Choose Executable',
      desc: 'Browse Finder or drag & drop a Windows .exe file or app folder above.',
      badge: 'Choose'
    },
    {
      num: '3',
      title: 'Choose Options',
      desc: 'Review app details. Swifter automatically picks the best graphics backend.',
      badge: 'Auto'
    },
    {
      num: '4',
      title: 'Pack & Run',
      desc: 'Click "Pack with Swifter". Your ready-to-run .app is saved to Library & Finder.',
      badge: 'Ready'
    }
  ];

  const tips = [
    {
      title: 'Installer or Setup Wizard?',
      desc: 'Click "Run Setup Wizard in Micro-Prefix" first, complete the installer, then pack the installed application.'
    },
    {
      title: 'Where is Save / App Data stored?',
      desc: 'Every app has its own isolated save and config folder. Access AppData and save files anytime from Library.'
    },
    {
      title: 'Which Graphics Backend to use?',
      desc: 'DXVK works best for DirectX 9–11 software and 3D titles. D3DMetal is recommended for DirectX 11/12 on Apple Silicon.'
    }
  ];
</script>

<div class="w-full max-w-2xl mx-auto flex flex-col space-y-5 pb-4 text-zinc-300">
  <!-- Guide Header -->
  <div class="border-b border-zinc-800/80 pb-3 space-y-1">
    <h2 class="text-xs font-semibold text-zinc-400 uppercase tracking-wider flex items-center gap-2">
      <BookOpen class="w-3.5 h-3.5 text-zinc-400" />
      How It Works
    </h2>
    <p class="text-xs text-zinc-400">
      4 simple steps to convert Windows executables and software into native macOS apps.
    </p>
  </div>

  <!-- Steps Grid -->
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
    {#each steps as step}
      <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/40 flex flex-col justify-between space-y-3">
        <div class="flex items-center justify-between">
          <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-mono text-xs font-bold text-zinc-100">
            {step.num}
          </div>
          <span class="text-[10px] font-mono uppercase px-2 py-0.5 rounded-full bg-zinc-800/80 border border-zinc-700/50 text-zinc-400">
            {step.badge}
          </span>
        </div>
        <div>
          <h3 class="text-xs font-semibold text-zinc-100 mb-1">{step.title}</h3>
          <p class="text-[11px] text-zinc-400 leading-relaxed">{step.desc}</p>
        </div>
      </div>
    {/each}
  </div>

  <!-- Quick Tips -->
  <div class="space-y-3 pt-2">
    <div class="text-xs font-semibold text-zinc-400 uppercase tracking-wider px-1 flex items-center gap-1.5">
      <HelpCircle class="w-3.5 h-3.5 text-zinc-400" />
      Quick Tips
    </div>

    <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/30 divide-y divide-zinc-800/60 text-xs">
      {#each tips as tip}
        <div class="py-2.5 first:pt-0 last:pb-0 space-y-1">
          <div class="font-medium text-zinc-200">{tip.title}</div>
          <p class="text-[11px] text-zinc-400 leading-relaxed">{tip.desc}</p>
        </div>
      {/each}
    </div>
  </div>
</div>
