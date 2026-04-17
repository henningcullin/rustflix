<script lang="ts">
  import FilmRow from '$lib/components/FilmRow.svelte';
  import EmptyLibrary from '$lib/components/EmptyLibrary.svelte';
  import { ChevronRight } from '$lib/lucide';
  import { listContinueWatching, listRecentlyAdded } from '$lib/api/films';
  import { alias, current } from '$lib/state/settings.svelte';
  import type { FilmListItem } from '$lib/types';
  import { onMount } from 'svelte';

  let continueWatching = $state<FilmListItem[]>([]);
  let recent = $state<FilmListItem[]>([]);
  let loaded = $state(false);

  const greeting = $derived(alias() ? `Welcome back, ${alias()}` : 'Your library');
  const hasAnyFilms = $derived(continueWatching.length > 0 || recent.length > 0);

  onMount(refresh);

  async function refresh() {
    loaded = false;
    try {
      const [cw, ra] = await Promise.all([
        listContinueWatching(10),
        listRecentlyAdded(10),
      ]);
      continueWatching = cw;
      recent = ra;
    } finally {
      loaded = true;
    }
    // ensure settings has been consumed for reactive greeting
    void current();
  }
</script>

<div class="p-6 max-w-6xl mx-auto space-y-8">
  <header>
    <h1 class="text-3xl font-bold">{greeting}</h1>
    <p class="text-sm text-muted-foreground mt-1">
      {hasAnyFilms ? 'Pick up where you left off or browse something new.' : 'Let\'s get your films set up.'}
    </p>
  </header>

  {#if !loaded}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if !hasAnyFilms}
    <EmptyLibrary onAdded={refresh} />
  {:else}
    {#if continueWatching.length > 0}
      <FilmRow title="Continue watching" films={continueWatching} />
    {/if}
    {#if recent.length > 0}
      <FilmRow title="Recently added" films={recent} />
    {/if}
    <div>
      <a href="/films" class="inline-flex items-center gap-1 text-sm font-medium hover:underline">
        Browse all films <ChevronRight class="size-4" />
      </a>
    </div>
  {/if}
</div>
