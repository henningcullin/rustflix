<script lang="ts">
  import {
    api,
    progressPct,
    type ContinueWatchingItem,
    type Movie,
    type Show,
  } from '$lib/api';
  import HeroBanner from '$lib/components/HeroBanner.svelte';
  import MediaRow from '$lib/components/MediaRow.svelte';
  import PosterCard from '$lib/components/PosterCard.svelte';
  import { FolderPlus } from '$lib/lucide';

  let movies: Movie[] = $state([]);
  let shows: Show[] = $state([]);
  let continueItems: ContinueWatchingItem[] = $state([]);
  let loading = $state(true);
  let hasLibraries = $state(true);

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    try {
      const libraries = await api.listLibraries();
      hasLibraries = libraries.length > 0;
      const [loadedMovies, loadedShows, loadedContinue] = await Promise.all([
        api.listMovies(),
        api.listShows(),
        api.continueWatching(),
      ]);
      movies = loadedMovies;
      shows = loadedShows;
      continueItems = loadedContinue;
    } catch (caught) {
      console.error(caught);
    } finally {
      loading = false;
    }
  }

  const featured = $derived.by<Movie | Show | null>(() => {
    if (movies.length > 0) {
      return [...movies].sort((a, b) => b.added_at - a.added_at)[0];
    }
    if (shows.length > 0) {
      return [...shows].sort((a, b) => b.added_at - a.added_at)[0];
    }
    return null;
  });

  const recentMovies = $derived([...movies].sort((a, b) => b.added_at - a.added_at).slice(0, 20));
  const recentShows = $derived([...shows].sort((a, b) => b.added_at - a.added_at).slice(0, 20));

  function showSubtitle(show: Show): string {
    const watched = show.watched_count;
    const total = show.episode_count;
    if (total === 0) {
      return show.year ? `${show.year}` : '';
    }
    return `${watched}/${total} watched${show.year ? ` · ${show.year}` : ''}`;
  }
</script>

{#if loading}
  <div class="flex h-[60vh] items-center justify-center text-muted-foreground">Loading…</div>
{:else if !hasLibraries}
  <div class="flex min-h-[70vh] flex-col items-center justify-center px-6 text-center">
    <div class="mb-4 rounded-full bg-primary/10 p-4">
      <FolderPlus class="size-10 text-primary" />
    </div>
    <h1 class="text-3xl font-bold tracking-tight">Welcome to Rustflix</h1>
    <p class="mt-2 max-w-md text-muted-foreground">
      Point Rustflix at a folder on your machine and it will index your movies and series.
    </p>
    <a
      href="/settings/libraries"
      class="mt-6 inline-flex items-center gap-2 rounded-md bg-primary px-5 py-2.5 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/30 transition hover:bg-primary/90"
    >
      Add a library
    </a>
  </div>
{:else if movies.length === 0 && shows.length === 0}
  <div class="flex min-h-[70vh] flex-col items-center justify-center px-6 text-center">
    <h1 class="text-2xl font-semibold">No media found</h1>
    <p class="mt-2 max-w-md text-muted-foreground">
      Your libraries are set up but no playable files were detected.
    </p>
    <a
      href="/settings/libraries"
      class="mt-6 inline-flex items-center gap-2 rounded-md bg-secondary px-5 py-2.5 text-sm font-semibold text-secondary-foreground transition hover:bg-accent"
    >
      Manage libraries
    </a>
  </div>
{:else}
  {#if featured}
    {@const isMovie = 'path' in featured && !('episode_count' in featured)}
    {@const detailHref = isMovie ? `/films/${featured.id}` : `/series/${featured.id}`}
    <HeroBanner
      title={featured.title}
      subtitle={isMovie ? 'Featured movie' : 'Featured series'}
      overview={featured.overview ?? null}
      href={detailHref}
      year={featured.year}
      runtime={isMovie ? (featured as Movie).duration_seconds : null}
      backdrop={featured.poster_path ?? null}
    />
  {/if}

  <div class="mt-8">
    {#if continueItems.length > 0}
      <MediaRow title="Continue Watching">
        {#each continueItems as item}
          {#if item.kind === 'movie'}
            <PosterCard
              href={`/films/${item.movie.id}`}
              title={item.movie.title}
              subtitle={item.movie.year ? String(item.movie.year) : undefined}
              posterPath={item.movie.poster_path}
              watched={item.movie.watched}
              progressPct={progressPct(item.movie)}
            />
          {:else}
            <PosterCard
              href={`/series/${item.show.id}`}
              title={item.show.title}
              subtitle={`S${String(item.episode.season).padStart(2, '0')}E${String(item.episode.episode).padStart(2, '0')} · ${item.episode.title}`}
              posterPath={item.show.poster_path}
              progressPct={progressPct(item.episode)}
            />
          {/if}
        {/each}
      </MediaRow>
    {/if}

    {#if recentMovies.length > 0}
      <MediaRow title="Recently Added Movies" href="/films">
        {#each recentMovies as movie (movie.id)}
          <PosterCard
            href={`/films/${movie.id}`}
            title={movie.title}
            subtitle={movie.year ? String(movie.year) : undefined}
            posterPath={movie.poster_path}
            watched={movie.watched}
            progressPct={progressPct(movie)}
          />
        {/each}
      </MediaRow>
    {/if}

    {#if recentShows.length > 0}
      <MediaRow title="Recently Added Series" href="/series">
        {#each recentShows as show (show.id)}
          <PosterCard
            href={`/series/${show.id}`}
            title={show.title}
            subtitle={showSubtitle(show)}
            posterPath={show.poster_path}
            watched={show.episode_count > 0 && show.watched_count === show.episode_count}
          />
        {/each}
      </MediaRow>
    {/if}
  </div>
{/if}
