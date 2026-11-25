<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { ExternalLink } from 'lucide-svelte';
  import { APP_VERSION } from '$lib/version';

  const credits = [
    {
      name: 'WineHQ & Gcenx',
      who: 'Dean M. & WineHQ',
      role: 'Windows compatibility layer and official macOS binary builds',
      url: 'https://github.com/Gcenx/macOS_Wine_builds'
    },
    {
      name: 'DXVK',
      who: 'Philip Rebohle',
      role: 'DirectX 9–11 to Vulkan translation layer',
      url: 'https://github.com/doitsujin/dxvk'
    },
    {
      name: 'MoltenVK',
      who: 'The Khronos Group',
      role: 'Vulkan graphics and compute driver over Apple Metal',
      url: 'https://github.com/KhronosGroup/MoltenVK'
    },
    {
      name: 'WineMetalGL',
      who: 'MetalSharp',
      role: 'OpenGL 1.0–4.6 to Apple Metal translation layer',
      url: 'https://github.com/MetalSharp/WineMetalGL'
    },
    {
      name: 'Tauri',
      who: 'Tauri Apps',
      role: 'Lightweight desktop harness and native OS bridge',
      url: 'https://github.com/tauri-apps/tauri'
    },
    {
      name: 'Rust',
      who: 'Rust Foundation',
      role: 'High-performance backend engine and PE binary analyzer',
      url: 'https://www.rust-lang.org'
    },
    {
      name: 'Svelte 5',
      who: 'Rich Harris & Contributors',
      role: 'Reactive frontend framework and UI state management',
      url: 'https://github.com/sveltejs/svelte'
    },
    {
      name: 'Tailwind CSS',
      who: 'Tailwind Labs',
      role: 'Utility-first styling and frosted glass design system',
      url: 'https://tailwindcss.com'
    }
  ];

  async function openLink(url: string) {
    try {
      await openUrl(url);
    } catch {
      window.open(url, '_blank');
    }
  }

  function openGithub() {
    openLink('https://github.com/msandypr/swifter');
  }

  function openAuthor() {
    openLink('https://github.com/msandypr');
  }
</script>

<div class="w-full flex flex-col space-y-4 pb-6 text-zinc-300">
  <!-- Header Card -->
  <div class="rounded-xl p-5 border border-white/[0.07] bg-white/[0.025] flex flex-col sm:flex-row items-center sm:items-start text-center sm:text-left gap-4">
    <img src="/logo-swifter.png" alt="Swifter" class="w-14 h-14 object-contain shrink-0" />

    <div class="flex-1 space-y-1.5">
      <div class="flex flex-col sm:flex-row sm:items-center gap-1.5">
        <h2 class="text-base font-bold text-zinc-100">Swifter</h2>
        <span class="text-[11px] font-mono text-zinc-400">v{APP_VERSION}</span>
      </div>

      <p class="text-xs text-zinc-400 leading-relaxed">
        A fast, lightweight utility for macOS that packages standalone Windows executables (.exe) and applications into native, ready-to-click macOS (.app) application bundles.
      </p>

      <div class="pt-1 flex flex-wrap items-center justify-center sm:justify-start gap-2">
        <button
          onclick={openAuthor}
          class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] border border-white/[0.08] text-xs font-medium text-zinc-200 transition-colors cursor-pointer"
        >
          <span>Created by <strong>msandypr</strong></span>
          <ExternalLink class="w-3 h-3 text-zinc-500" />
        </button>

        <button
          onclick={openGithub}
          class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] border border-white/[0.08] text-xs font-medium text-zinc-200 transition-colors cursor-pointer"
        >
          <svg class="w-3.5 h-3.5 text-zinc-400" viewBox="0 0 24 24" fill="currentColor">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
          </svg>
          <span>GitHub Repository</span>
          <ExternalLink class="w-3 h-3 text-zinc-500" />
        </button>

        <button
          onclick={openGithub}
          class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] border border-white/[0.08] text-xs font-medium text-zinc-300 transition-colors cursor-pointer"
        >
          <span>GPL-3.0 License</span>
          <ExternalLink class="w-3 h-3 text-zinc-500" />
        </button>
      </div>
    </div>
  </div>

  <!-- Open-Source Upstream Credits -->
  <div class="space-y-2">
    <div class="text-xs font-semibold text-zinc-400 uppercase tracking-wider px-1">
      Open-Source Credits
    </div>

    <div class="rounded-xl p-4 border border-white/[0.07] bg-white/[0.025] divide-y divide-white/[0.06]">
      {#each credits as { name, who, role, url }}
        <div class="flex items-center justify-between py-2.5 first:pt-0.5 last:pb-0.5 gap-3">
          <div class="min-w-0 space-y-0.5">
            <div class="flex items-center gap-2 text-xs">
              <span class="font-medium text-zinc-100">{name}</span>
              <span class="text-zinc-500 font-mono text-[11px] truncate">by {who}</span>
            </div>
            <p class="text-[11px] text-zinc-400 leading-relaxed truncate">{role}</p>
          </div>
          <button
            onclick={() => openLink(url)}
            class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/80 transition-colors cursor-pointer active:scale-95 shrink-0"
            title="Visit {name}"
          >
            <ExternalLink class="w-3.5 h-3.5" />
          </button>
        </div>
      {/each}
    </div>
  </div>
</div>
