<script lang="ts">
  import { api, type Show } from '$lib/api';
  import PosterCard from '$lib/components/PosterCard.svelte';
  import { Search } from '$lib/lucide';

  let shows: Show[] = $state([]);
  let loading = $state(true);
  let query = $state('');

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    try {
      shows = await api.listShows();
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  const filtered = $derived(
    query.trim() === ''
      ? shows
      : shows.filter((s) => s.title.toLowerCase().includes(query.toLowerCase())),
  );

  function subtitle(s: Show): string {
    if (s.episode_count === 0) return s.year ? String(s.year) : '';
    return `${s.watched_count}/${s.episode_count} watched`;
  }
</script>

<div class="px-6 py-8 lg:px-12">
  <header class="mb-6 flex flex-wrap items-end justify-between gap-4">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Series</h1>
      <p class="text-muted-foreground">{shows.length} shows</p>
    </div>
    <div class="relative">
      <Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
      <input
        type="search"
        bind:value={query}
        placeholder="Search series…"
        class="w-72 rounded-md border border-border bg-background py-2 pl-9 pr-3 text-sm placeholder:text-muted-foreground focus:border-primary focus:outline-none"
      />
    </div>
  </header>

  {#if loading}
    <p class="text-muted-foreground">Loading…</p>
  {:else if filtered.length === 0}
    <p class="text-muted-foreground">No series match your search.</p>
  {:else}
    <div
      class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7"
    >
      {#each filtered as show (show.id)}
        <PosterCard
          href={`/series/${show.id}`}
          title={show.title}
          subtitle={subtitle(show)}
          posterPath={show.poster_path}
          watched={show.episode_count > 0 && show.watched_count === show.episode_count}
        />
      {/each}
    </div>
  {/if}
</div>
