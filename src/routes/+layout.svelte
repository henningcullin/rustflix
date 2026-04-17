<script lang="ts">
  import * as Sidebar from '$lib/components/ui/sidebar/index';
  import AppSidebar from '$lib/components/AppSidebar.svelte';
  import { loadSettings, firstRunCompleted } from '$lib/state/settings.svelte';
  import { applyTheme } from '$lib/state/theme.svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';

  import '../app.css';

  let { children } = $props();

  let ready = $state(false);

  onMount(async () => {
    const s = await loadSettings();
    applyTheme(s?.theme ?? 'system');
    ready = true;
    if (!firstRunCompleted() && $page.url.pathname !== '/welcome') {
      await goto('/welcome', { replaceState: true });
    }
  });

  const isWizard = $derived($page.url.pathname === '/welcome');
</script>

{#if isWizard}
  {#if ready}
    {@render children()}
  {/if}
{:else}
  <Sidebar.Provider open={false}>
    <AppSidebar />
    <main class="flex-1 min-w-0">
      <Sidebar.Trigger />
      {#if ready}
        {@render children()}
      {/if}
    </main>
  </Sidebar.Provider>
{/if}
