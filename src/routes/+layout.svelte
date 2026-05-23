<script lang="ts">
  import TopNav from '$lib/components/TopNav.svelte';
  import '../app.css';

  let { children } = $props();
</script>

<div class="min-h-screen bg-background text-foreground">
  <TopNav />
  <main class="mx-auto w-full max-w-[1600px]">
    {@render children()}
  </main>
</div>
