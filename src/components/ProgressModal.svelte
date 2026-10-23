<script lang="ts">
  import { CheckCircle2, PackageCheck, Rocket, FolderOpen, X, AlertCircle } from 'lucide-svelte';

  let {
    isOpen = false,
    isComplete = false,
    error = '',
    bundlePath = '',
    appName = '',
    currentStep = 1,
    onClose,
    onLaunch,
    onReveal
  }: {
    isOpen: boolean;
    isComplete: boolean;
    error?: string;
    bundlePath?: string;
    appName?: string;
    currentStep?: number;
    onClose: () => void;
    onLaunch?: (path: string) => void;
    onReveal?: (path: string) => void;
  } = $props();

  const steps = [
    { num: 1, label: 'Preparing game files & assets' },
    { num: 2, label: 'Configuring compatibility engine' },
    { num: 3, label: 'Building native macOS application' },
    { num: 4, label: 'Finalizing ready-to-play package' },
  ];
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md">
    <div class="w-full max-w-md glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-6">
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 text-zinc-200 flex items-center justify-center">
            {#if error}
              <AlertCircle class="w-5 h-5 text-rose-400" />
            {:else if isComplete}
              <PackageCheck class="w-5 h-5 text-zinc-100" />
            {:else}
              <div class="w-4 h-4 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
            {/if}
          </div>
          <div>
            <h3 class="text-sm font-semibold text-zinc-100">
              {error ? 'Packaging Unsuccessful' : isComplete ? 'Packaging Complete' : 'Packaging Application'}
            </h3>
            <p class="text-xs text-zinc-400">
              {error ? 'An issue occurred during packaging.' : isComplete ? `${appName} is ready to play.` : 'Creating your native macOS app.'}
            </p>
          </div>
        </div>

        {#if isComplete || error}
          <button onclick={onClose} class="text-zinc-400 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800 transition-colors">
            <X class="w-4 h-4" />
          </button>
        {/if}
      </div>

      <!-- Step List (If no error) -->
      {#if !error}
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
      {:else}
        <!-- Simple Friendly Error Message -->
        <div class="p-4 rounded-xl bg-zinc-900 border border-rose-500/30 text-xs space-y-2 text-rose-300">
          <div class="font-medium text-rose-200">Unable to create application</div>
          <p class="text-[11px] text-zinc-300 leading-relaxed">
            Please verify that the game folder is accessible and not currently in use by another program.
          </p>
          {#if error}
            <div class="font-mono text-[10px] text-zinc-500 bg-zinc-950 p-2 rounded border border-zinc-800 break-all select-all">
              {error}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Result Banner & Actions -->
      {#if isComplete && !error}
        <div class="p-3 rounded-xl bg-zinc-900 border border-zinc-800 space-y-1">
          <div class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider">Bundle Location</div>
          <div class="text-xs font-mono text-zinc-300 break-all select-all">{bundlePath}</div>
        </div>

        <div class="flex items-center gap-2 pt-2">
          {#if onLaunch}
            <button
              onclick={() => onLaunch(bundlePath)}
              class="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 transition-all shadow-sm cursor-pointer"
            >
              <Rocket class="w-3.5 h-3.5" />
              Launch Game
            </button>
          {/if}

          {#if onReveal}
            <button
              onclick={() => onReveal(bundlePath)}
              class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-xs font-medium text-zinc-200 border border-zinc-700 transition-all cursor-pointer"
            >
              <FolderOpen class="w-3.5 h-3.5" />
              In Finder
            </button>
          {/if}

          <button
            onclick={onClose}
            class="px-4 py-2.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-medium text-zinc-400 transition-colors cursor-pointer"
          >
            Close
          </button>
        </div>
      {:else if error}
        <div class="flex justify-end pt-1">
          <button
            onclick={onClose}
            class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-200 transition-colors cursor-pointer"
          >
            Close
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
