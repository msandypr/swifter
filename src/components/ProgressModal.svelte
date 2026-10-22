<script lang="ts">
  import { CheckCircle2, PackageCheck, Play, FolderOpen, X } from 'lucide-svelte';

  let {
    isOpen = false,
    isComplete = false,
    bundlePath = '',
    appName = '',
    currentStep = 1,
    onClose,
    onLaunch,
    onReveal
  }: {
    isOpen: boolean;
    isComplete: boolean;
    bundlePath?: string;
    appName?: string;
    currentStep?: number;
    onClose: () => void;
    onLaunch?: (path: string) => void;
    onReveal?: (path: string) => void;
  } = $props();

  const steps = [
    { num: 1, label: 'Reading PE binary & indie assets' },
    { num: 2, label: 'Provisioning isolated micro-prefix (~15MB)' },
    { num: 3, label: 'Generating POSIX launcher & translation config' },
    { num: 4, label: 'Assembling macOS .app bundle & permissions' },
  ];
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md">
    <div class="w-full max-w-md glass-panel rounded-2xl p-6 shadow-2xl border border-white/15 space-y-6 animate-in fade-in zoom-in-95 duration-200">
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl {isComplete ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-sky-500/20 text-sky-400 border border-sky-500/30'} flex items-center justify-center">
            {#if isComplete}
              <PackageCheck class="w-5 h-5" />
            {:else}
              <div class="w-5 h-5 border-2 border-sky-400 border-t-transparent rounded-full animate-spin"></div>
            {/if}
          </div>
          <div>
            <h3 class="text-base font-semibold text-slate-100">
              {isComplete ? 'Packaging Complete!' : 'Packaging Application...'}
            </h3>
            <p class="text-xs text-slate-400">
              {isComplete ? `${appName} is ready to run on macOS.` : 'Creating optimized native application bundle.'}
            </p>
          </div>
        </div>

        {#if isComplete}
          <button onclick={onClose} class="text-slate-400 hover:text-slate-200 p-1 rounded-lg hover:bg-white/5 transition-colors">
            <X class="w-5 h-5" />
          </button>
        {/if}
      </div>

      <!-- Step List -->
      <div class="space-y-3 py-2">
        {#each steps as step}
          <div class="flex items-center gap-3 text-xs">
            {#if isComplete || currentStep > step.num}
              <div class="w-5 h-5 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center shrink-0">
                <CheckCircle2 class="w-4 h-4" />
              </div>
              <span class="text-slate-300 line-through opacity-70">{step.label}</span>
            {:else if currentStep === step.num}
              <div class="w-5 h-5 rounded-full bg-sky-500/20 text-sky-400 flex items-center justify-center shrink-0">
                <div class="w-2.5 h-2.5 border-2 border-sky-400 border-t-transparent rounded-full animate-spin"></div>
              </div>
              <span class="text-slate-100 font-medium">{step.label}</span>
            {:else}
              <div class="w-5 h-5 rounded-full bg-white/5 text-slate-500 flex items-center justify-center shrink-0 font-mono text-[10px]">
                {step.num}
              </div>
              <span class="text-slate-500">{step.label}</span>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Result Banner & Actions -->
      {#if isComplete}
        <div class="p-3 rounded-xl bg-white/5 border border-white/10 space-y-1">
          <div class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Bundle Location</div>
          <div class="text-xs font-mono text-slate-200 break-all select-all">{bundlePath}</div>
        </div>

        <div class="flex items-center gap-2 pt-2">
          {#if onLaunch}
            <button
              onclick={() => onLaunch(bundlePath)}
              class="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-emerald-500 to-teal-600 hover:from-emerald-400 hover:to-teal-500 active:scale-95 text-xs font-semibold text-white shadow-lg shadow-emerald-600/25 transition-all"
            >
              <Play class="w-4 h-4" />
              Launch Game
            </button>
          {/if}

          {#if onReveal}
            <button
              onclick={() => onReveal(bundlePath)}
              class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-white/10 hover:bg-white/15 active:scale-95 text-xs font-medium text-slate-200 border border-white/10 transition-all"
            >
              <FolderOpen class="w-4 h-4" />
              In Finder
            </button>
          {/if}

          <button
            onclick={onClose}
            class="px-4 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-xs font-medium text-slate-300 transition-colors"
          >
            Close
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
