<script lang="ts">
  import { api, progressPct, type Movie } from '$lib/api';
  import PosterCard from '$lib/components/PosterCard.svelte';
  import { Search } from '$lib/lucide';

  let movies: Movie[] = $state([]);
  let loading = $state(true);
  let query = $state('');

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    try {
      movies = await api.listMovies();
    } catch (caught) {
      console.error(caught);
    } finally {
      loading = false;
    }
  }

  const filtered = $derived(
    query.trim() === ''
      ? movies
      : movies.filter((movie) => movie.title.toLowerCase().includes(query.toLowerCase())),
  );
</script>

<div class="px-6 py-8 lg:px-12">
  <header class="mb-6 flex flex-wrap items-end justify-between gap-4">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Movies</h1>
      <p class="text-muted-foreground">{movies.length} titles</p>
    </div>
    <div class="relative">
      <Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
      <input
        type="search"
        bind:value={query}
        placeholder="Search movies…"
        class="w-72 rounded-md border border-border bg-background py-2 pl-9 pr-3 text-sm placeholder:text-muted-foreground focus:border-primary focus:outline-none"
      />
    </div>
  </header>

  {#if loading}
    <p class="text-muted-foreground">Loading…</p>
  {:else if filtered.length === 0}
    <p class="text-muted-foreground">No movies match your search.</p>
  {:else}
    <div
      class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7"
    >
      {#each filtered as movie (movie.id)}
        <PosterCard
          href={`/films/${movie.id}`}
          title={movie.title}
          subtitle={movie.year ? String(movie.year) : undefined}
          posterPath={movie.poster_path}
          watched={movie.watched}
          progressPct={progressPct(movie)}
        />
      {/each}
    </div>
  {/if}
</div>
