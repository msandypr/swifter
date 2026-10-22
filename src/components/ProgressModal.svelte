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
    <div class="w-full max-w-md glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-6">
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 text-zinc-200 flex items-center justify-center">
            {#if isComplete}
              <PackageCheck class="w-5 h-5" />
            {:else}
              <div class="w-4 h-4 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
            {/if}
          </div>
          <div>
            <h3 class="text-sm font-semibold text-zinc-100">
              {isComplete ? 'Packaging Complete' : 'Packaging Application'}
            </h3>
            <p class="text-xs text-zinc-400">
              {isComplete ? `${appName} is ready to run on macOS.` : 'Creating optimized native application bundle.'}
            </p>
          </div>
        </div>

        {#if isComplete}
          <button onclick={onClose} class="text-zinc-400 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800 transition-colors">
            <X class="w-4 h-4" />
          </button>
        {/if}
      </div>

      <!-- Step List -->
      <div class="space-y-3 py-1">
        {#each steps as step}
          <div class="flex items-center gap-3 text-xs">
            {#if isComplete || currentStep > step.num}
              <div class="w-5 h-5 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-200 flex items-center justify-center shrink-0">
                <CheckCircle2 class="w-3.5 h-3.5" />
              </div>
              <span class="text-zinc-400 line-through opacity-70">{step.label}</span>
            {:else if currentStep === step.num}
              <div class="w-5 h-5 rounded-full bg-zinc-800 border border-zinc-500 text-zinc-100 flex items-center justify-center shrink-0">
                <div class="w-2.5 h-2.5 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
              </div>
              <span class="text-zinc-100 font-medium">{step.label}</span>
            {:else}
              <div class="w-5 h-5 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-600 flex items-center justify-center shrink-0 font-mono text-[10px]">
                {step.num}
              </div>
              <span class="text-zinc-500">{step.label}</span>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Result Banner & Actions -->
      {#if isComplete}
        <div class="p-3 rounded-xl bg-zinc-900 border border-zinc-800 space-y-1">
          <div class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider">Bundle Location</div>
          <div class="text-xs font-mono text-zinc-300 break-all select-all">{bundlePath}</div>
        </div>

        <div class="flex items-center gap-2 pt-2">
          {#if onLaunch}
            <button
              onclick={() => onLaunch(bundlePath)}
              class="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 transition-all shadow-sm"
            >
              <Play class="w-3.5 h-3.5" />
              Launch Game
            </button>
          {/if}

          {#if onReveal}
            <button
              onclick={() => onReveal(bundlePath)}
              class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-xs font-medium text-zinc-200 border border-zinc-700 transition-all"
            >
              <FolderOpen class="w-3.5 h-3.5" />
              In Finder
            </button>
          {/if}

          <button
            onclick={onClose}
            class="px-4 py-2.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-medium text-zinc-400 transition-colors"
          >
            Close
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
