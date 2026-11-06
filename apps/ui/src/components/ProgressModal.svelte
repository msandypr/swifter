<script lang="ts">
  import { CheckCircle2, Rocket, FolderOpen, X, AlertCircle } from 'lucide-svelte';

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

  const stages: Record<number, { title: string; desc: string }> = {
    1: {
      title: 'Preparing application assets',
      desc: 'Inspecting binaries and extracting application icons...'
    },
    2: {
      title: 'Configuring translation engine',
      desc: 'Setting up graphics layer overrides and isolated micro-prefix...'
    },
    3: {
      title: 'Building native macOS bundle',
      desc: 'Writing launcher shim, Info.plist, and bundle resources...'
    },
    4: {
      title: 'Finalizing application package',
      desc: 'Configuring permissions and registering application bundle...'
    }
  };

  let progressPercent = $derived(
    isComplete ? 100 : Math.min(Math.max(currentStep * 25, 10), 95)
  );
  let activeStage = $derived(stages[currentStep] || stages[1]);
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md">
    <div class="w-full max-w-lg glass-panel rounded-2xl p-6 shadow-2xl border border-zinc-800 space-y-5 animate-in fade-in zoom-in-95 duration-200">
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center text-zinc-200 shrink-0">
            {#if error}
              <AlertCircle class="w-5 h-5 text-rose-400" />
            {:else if isComplete}
              <CheckCircle2 class="w-5 h-5 text-emerald-400" />
            {:else}
              <div class="w-4 h-4 border-2 border-zinc-300 border-t-transparent rounded-full animate-spin"></div>
            {/if}
          </div>
          <div class="min-w-0">
            <h3 class="text-sm font-semibold text-zinc-100 truncate">
              {#if error}
                Packaging Failed
              {:else if isComplete}
                {appName || 'Application'} Ready
              {:else}
                Packaging {appName || 'Application'}
              {/if}
            </h3>
            <p class="text-xs text-zinc-400">
              {#if error}
                An issue occurred while creating the application bundle.
              {:else if isComplete}
                Application bundle successfully built.
              {:else}
                Creating native macOS application bundle...
              {/if}
            </p>
          </div>
        </div>

        {#if isComplete || error}
          <button
            onclick={onClose}
            class="text-zinc-400 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        {/if}
      </div>

      {#if !isComplete && !error}
        <!-- Real-Time Progress Track -->
        <div class="space-y-3 p-4 rounded-xl bg-zinc-950/80 border border-zinc-800/80">
          <div class="space-y-1.5">
            <div class="flex items-center justify-between text-xs text-zinc-300">
              <span class="font-medium truncate pr-2">
                {activeStage.title} ({progressPercent}%)
              </span>
              <span class="font-mono text-[11px] text-zinc-400 shrink-0">
                Step {currentStep} of 4
              </span>
            </div>

            <!-- Bar Track -->
            <div class="w-full h-2 bg-zinc-800 rounded-full overflow-hidden">
              <div
                class="h-full bg-zinc-200 rounded-full transition-all duration-300"
                style="width: {progressPercent}%"
              ></div>
            </div>
          </div>

          <!-- Details Row -->
          <div class="flex items-center justify-between text-[11px] text-zinc-500 pt-0.5">
            <span class="font-mono truncate">
              {activeStage.desc}
            </span>
          </div>
        </div>
      {:else if isComplete && !error}
        <!-- Success State -->
        <div class="p-4 rounded-xl bg-zinc-900 border border-zinc-800 space-y-2 text-xs text-zinc-300">
          <div class="font-semibold text-zinc-100 flex items-center gap-1.5">
            <CheckCircle2 class="w-4 h-4 text-emerald-400" />
            Bundle Created Successfully
          </div>
          <div class="font-mono text-[11px] text-zinc-400 bg-zinc-950 px-2.5 py-1.5 rounded-lg border border-zinc-800 break-all select-all">
            {bundlePath}
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center gap-2 pt-1">
          {#if onLaunch}
            <button
              onclick={() => onLaunch(bundlePath)}
              class="flex-1 inline-flex items-center justify-center gap-1.5 px-4 py-2.5 rounded-xl bg-zinc-100 hover:bg-white active:scale-95 text-xs font-semibold text-zinc-950 transition-all shadow-sm cursor-pointer"
            >
              <Rocket class="w-3.5 h-3.5" />
              <span>Launch</span>
            </button>
          {/if}

          {#if onReveal}
            <button
              onclick={() => onReveal(bundlePath)}
              class="inline-flex items-center justify-center gap-1.5 px-4 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 active:scale-95 text-xs font-medium text-zinc-200 border border-zinc-700 transition-all cursor-pointer"
            >
              <FolderOpen class="w-3.5 h-3.5" />
              <span>In Finder</span>
            </button>
          {/if}

          <button
            onclick={onClose}
            class="px-4 py-2.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-medium text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
          >
            Done
          </button>
        </div>
      {:else if error}
        <!-- Error State -->
        <div class="p-4 rounded-xl bg-zinc-900 border border-rose-500/30 text-xs space-y-2 text-rose-300">
          <div class="font-medium text-rose-200">Packaging Encountered an Error</div>
          <p class="text-[11px] text-zinc-300 leading-relaxed">
            Please verify that the application directory is accessible and not locked.
          </p>
          <div class="font-mono text-[10px] text-zinc-400 bg-zinc-950 p-2 rounded-lg border border-zinc-800 break-all select-all">
            {error}
          </div>
        </div>

        <div class="flex justify-end pt-1">
          <button
            onclick={onClose}
            class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium transition-all cursor-pointer"
          >
            Close
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
