<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { Info, ExternalLink, Heart, Sparkles } from 'lucide-svelte';

  async function openGithub() {
    try {
      await openUrl('https://github.com/msandypr/swifter');
    } catch {
      window.open('https://github.com/msandypr/swifter', '_blank');
    }
  }

  async function openAuthor() {
    try {
      await openUrl('https://github.com/msandypr');
    } catch {
      window.open('https://github.com/msandypr', '_blank');
    }
  }
</script>

<div class="w-full max-w-2xl mx-auto flex flex-col p-6 space-y-6 overflow-y-auto max-h-[calc(100vh-4rem)] text-zinc-300">
  <!-- Header Card -->
  <div class="glass-card rounded-2xl p-6 border border-zinc-800 bg-zinc-900/40 flex flex-col items-center text-center space-y-3">
    <img src="/logo-swifter.png" alt="Swifter" class="w-16 h-16 rounded-2xl shadow-md object-contain" />

    <div>
      <h2 class="text-xl font-bold text-zinc-100">Swifter</h2>
      <div class="text-xs font-mono text-zinc-400 mt-0.5">Version 0.1.0</div>
    </div>

    <p class="text-xs text-zinc-400 max-w-md leading-relaxed">
      A fast, lightweight utility for macOS that packages standalone Windows executables (.exe) and DRM-free indie games into native, ready-to-click macOS (.app) application bundles.
    </p>

    <!-- Author Badge -->
    <div class="pt-2">
      <button
        onclick={openAuthor}
        class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-xs font-medium text-zinc-200 transition-all active:scale-95"
      >
        <Heart class="w-3.5 h-3.5 text-zinc-400" />
        <span>Created by <strong>msandypr</strong></span>
        <ExternalLink class="w-3 h-3 text-zinc-500" />
      </button>
    </div>
  </div>

  <!-- Links & Repository -->
  <div class="glass-card rounded-2xl p-4 border border-zinc-800 bg-zinc-900/40 flex items-center justify-between">
    <div class="flex items-center gap-3">
      <svg class="w-5 h-5 text-zinc-400" viewBox="0 0 24 24" fill="currentColor">
        <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
      </svg>
      <div class="text-xs">
        <div class="font-semibold text-zinc-200">Open Source Repository</div>
        <div class="text-zinc-500 font-mono text-[11px]">github.com/msandypr/swifter</div>
      </div>
    </div>

    <button
      onclick={openGithub}
      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-xs font-medium text-zinc-200 border border-zinc-700 transition-all"
    >
      <span>GitHub</span>
      <ExternalLink class="w-3.5 h-3.5 text-zinc-400" />
    </button>
  </div>
</div>
