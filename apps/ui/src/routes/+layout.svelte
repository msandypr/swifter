<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<div class="h-screen w-screen bg-[#111113] text-zinc-100 overflow-hidden font-sans select-none flex">
  {@render children()}
</div>
