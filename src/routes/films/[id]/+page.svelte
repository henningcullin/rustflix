<script lang="ts">
  import { page } from '$app/stores';
  import { api, formatRuntime, progressPct, type Movie } from '$lib/api';
  import HeroBanner from '$lib/components/HeroBanner.svelte';
  import { Play, Check, Circle } from '$lib/lucide';

  let movie: Movie | null = $state(null);
  let loading = $state(true);
  let playing = $state(false);
  let error = $state<string | null>(null);

  const id = $derived(Number($page.params.id));

  $effect(() => {
    if (!Number.isFinite(id)) return;
    void load(id);
  });

  async function load(movieId: number) {
    loading = true;
    error = null;
    try {
      movie = await api.getMovie(movieId);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function play(fromStart = false) {
    if (!movie) return;
    playing = true;
    try {
      await api.playMovie(movie.id, fromStart ? 0 : undefined);
      await load(movie.id);
    } catch (e) {
      error = String(e);
    } finally {
      playing = false;
    }
  }

  async function toggleWatched() {
    if (!movie) return;
    try {
      await api.setWatched('movie', movie.id, !movie.watched);
      await load(movie.id);
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if loading}
  <div class="flex h-[60vh] items-center justify-center text-muted-foreground">Loading…</div>
{:else if error || !movie}
  <div class="px-6 py-12 text-destructive-foreground">{error ?? 'Movie not found.'}</div>
{:else}
  <HeroBanner
    title={movie.title}
    subtitle="Movie"
    overview={movie.overview ?? null}
    href={`/films/${movie.id}`}
    year={movie.year}
    runtime={movie.duration_seconds}
    backdrop={movie.poster_path ?? null}
  />

  <div class="px-6 py-8 lg:px-12">
    <div class="mb-6 flex flex-wrap items-center gap-3">
      <button
        type="button"
        onclick={() => play(false)}
        disabled={playing}
        class="inline-flex items-center gap-2 rounded-md bg-primary px-5 py-2.5 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/30 transition hover:bg-primary/90 disabled:opacity-50"
      >
        <Play class="size-4 fill-current" />
        {movie.progress_seconds > 30 && !movie.watched
          ? `Resume at ${formatRuntime(movie.progress_seconds)}`
          : 'Play'}
      </button>
      {#if movie.progress_seconds > 30 && !movie.watched}
        <button
          type="button"
          onclick={() => play(true)}
          disabled={playing}
          class="inline-flex items-center gap-2 rounded-md bg-secondary px-4 py-2.5 text-sm font-semibold text-secondary-foreground transition hover:bg-accent disabled:opacity-50"
        >
          Play from start
        </button>
      {/if}
      <button
        type="button"
        onclick={toggleWatched}
        class="inline-flex items-center gap-2 rounded-md border border-border bg-background px-4 py-2.5 text-sm font-semibold transition hover:bg-accent"
      >
        {#if movie.watched}
          <Check class="size-4 text-emerald-400" /> Watched
        {:else}
          <Circle class="size-4" /> Mark watched
        {/if}
      </button>
    </div>

    {#if !movie.watched && movie.progress_seconds > 0 && movie.duration_seconds}
      <div class="mb-6 max-w-md">
        <div class="mb-1 flex justify-between text-xs text-muted-foreground">
          <span>{formatRuntime(movie.progress_seconds)} watched</span>
          <span>{formatRuntime(movie.duration_seconds)}</span>
        </div>
        <div class="h-1.5 overflow-hidden rounded-full bg-muted">
          <div class="h-full bg-primary" style="width: {progressPct(movie)}%"></div>
        </div>
      </div>
    {/if}

    <dl class="grid max-w-2xl grid-cols-[auto_1fr] gap-x-6 gap-y-2 text-sm">
      {#if movie.year}
        <dt class="text-muted-foreground">Year</dt>
        <dd>{movie.year}</dd>
      {/if}
      <dt class="text-muted-foreground">File</dt>
      <dd class="break-all font-mono text-xs">{movie.path}</dd>
    </dl>
  </div>
{/if}
