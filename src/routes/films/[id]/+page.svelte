<script lang="ts">
  import { page } from '$app/stores';
  import { getFilm } from '$lib/api/films';
  import Player from '$lib/player/Player.svelte';
  import * as Card from '$lib/components/ui/card/index';
  import type { FilmDetail } from '$lib/types';
  import { onMount } from 'svelte';

  let film = $state<FilmDetail | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const filmId = $derived(Number($page.params.id));

  onMount(() => {
    void load();
  });

  async function load() {
    loading = true;
    error = null;
    try {
      film = await getFilm(filmId);
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  function posterUrl(f: FilmDetail): string | null {
    if (!f.poster_path) return null;
    if (f.poster_path.startsWith('/')) {
      return `https://image.tmdb.org/t/p/w500${f.poster_path}`;
    }
    return null;
  }

  function formatRuntime(minutes: number | null): string | null {
    if (!minutes) return null;
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return h > 0 ? `${h}h ${m}m` : `${m}m`;
  }

  const directors = $derived(film?.cast.filter((c) => c.role === 'director') ?? []);
  const actors = $derived(film?.cast.filter((c) => c.role === 'actor') ?? []);
</script>

<div class="p-6 max-w-6xl mx-auto space-y-6">
  {#if error}
    <p class="text-red-600 text-sm">{error}</p>
  {/if}

  {#if loading && !film}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if film}
    <div>
      <h1 class="text-2xl font-bold mb-1">{film.title}</h1>
      {#if film.original_title && film.original_title !== film.title}
        <p class="text-sm text-muted-foreground">{film.original_title}</p>
      {/if}
    </div>

    <Player filmId={film.id} resumeFrom={film.left_off_point} />

    <div class="grid md:grid-cols-[200px_1fr] gap-6">
      <div class="aspect-[2/3] bg-muted rounded-md overflow-hidden">
        {#if posterUrl(film)}
          <img
            src={posterUrl(film)}
            alt={film.title}
            class="w-full h-full object-cover"
          />
        {/if}
      </div>

      <div class="space-y-4">
        <div class="flex flex-wrap gap-3 text-sm text-muted-foreground">
          {#if film.release_date}
            <span>{film.release_date.slice(0, 4)}</span>
          {/if}
          {#if formatRuntime(film.runtime)}
            <span>· {formatRuntime(film.runtime)}</span>
          {/if}
          {#if film.rating !== null}
            <span>· ★ {film.rating.toFixed(1)}</span>
          {/if}
        </div>

        {#if film.genres.length > 0}
          <div class="flex flex-wrap gap-2">
            {#each film.genres as g (g.id)}
              <span class="text-xs px-2 py-1 rounded bg-muted">{g.name}</span>
            {/each}
          </div>
        {/if}

        {#if film.overview}
          <p class="text-sm leading-relaxed">{film.overview}</p>
        {/if}

        {#if directors.length > 0}
          <Card.Root>
            <Card.Header>
              <Card.Title class="text-base">
                {directors.length > 1 ? 'Directors' : 'Director'}
              </Card.Title>
            </Card.Header>
            <Card.Content>
              <p class="text-sm">{directors.map((d) => d.name).join(', ')}</p>
            </Card.Content>
          </Card.Root>
        {/if}

        {#if actors.length > 0}
          <Card.Root>
            <Card.Header>
              <Card.Title class="text-base">Cast</Card.Title>
            </Card.Header>
            <Card.Content>
              <ul class="grid grid-cols-2 sm:grid-cols-3 gap-2 text-sm">
                {#each actors.slice(0, 12) as c (c.person_id)}
                  <li class="min-w-0">
                    <div class="font-medium truncate">{c.name}</div>
                    {#if c.character}
                      <div class="text-xs text-muted-foreground truncate">
                        {c.character}
                      </div>
                    {/if}
                  </li>
                {/each}
              </ul>
            </Card.Content>
          </Card.Root>
        {/if}

        <p class="text-xs text-muted-foreground break-all">
          {film.file_path}
        </p>
      </div>
    </div>
  {/if}
</div>
