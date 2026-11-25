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

<div class="w-full flex flex-col space-y-4 pb-2 text-zinc-300">
  <!-- Guide Header -->
  <div class="flex items-center justify-between border-b border-white/[0.06] pb-2">
    <h2 class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider flex items-center gap-1.5">
      <BookOpen class="w-3.5 h-3.5 text-zinc-400" />
      How It Works
    </h2>
    <span class="text-[11px] text-zinc-500">4 steps to package Windows apps</span>
  </div>

  <!-- Steps Grid -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2.5">
    {#each steps as step}
      <div class="rounded-xl p-3.5 border border-white/[0.07] bg-white/[0.02] flex flex-col justify-between space-y-2.5">
        <div class="flex items-center justify-between">
          <div class="w-6 h-6 rounded-md bg-zinc-800 border border-white/[0.08] flex items-center justify-center font-mono text-[11px] font-bold text-zinc-100">
            {step.num}
          </div>
          <span class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-zinc-800/80 border border-white/[0.06] text-zinc-400">
            {step.badge}
          </span>
        </div>
        <div>
          <h3 class="text-xs font-semibold text-zinc-100 mb-0.5">{step.title}</h3>
          <p class="text-[11px] text-zinc-400 leading-relaxed">{step.desc}</p>
        </div>
      </div>
    {/each}
  </div>

  <!-- Quick Tips -->
  <div class="space-y-2 pt-1">
    <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider flex items-center gap-1.5">
      <HelpCircle class="w-3.5 h-3.5 text-zinc-400" />
      Quick Tips
    </div>

    <div class="rounded-xl p-3.5 border border-white/[0.07] bg-white/[0.02] divide-y divide-white/[0.06] text-xs">
      {#each tips as tip}
        <div class="py-2 first:pt-0 last:pb-0 space-y-0.5">
          <div class="font-medium text-zinc-200">{tip.title}</div>
          <p class="text-[11px] text-zinc-400 leading-relaxed">{tip.desc}</p>
        </div>
      {/each}
    </div>
  </div>
</div>
