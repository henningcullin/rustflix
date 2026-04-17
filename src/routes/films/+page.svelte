<script lang="ts">
  import { listFilms } from '$lib/api/films';
  import type { FilmListItem } from '$lib/types';
  import * as Card from '$lib/components/ui/card/index';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let films = $state<FilmListItem[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    loading = true;
    try {
      films = await listFilms();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  });

  function posterUrl(film: FilmListItem): string | null {
    if (!film.poster_path) return null;
    // poster_path is stored as "covers/{film_id}" referring to a directory
    // under app data; we use asset protocol via convertFileSrc on the full path.
    // For M1 we show a simple placeholder if cover path is not a TMDb URL.
    if (film.poster_path.startsWith('/')) {
      return `https://image.tmdb.org/t/p/w500${film.poster_path}`;
    }
    return null;
  }

  function formatDuration(seconds: number | null) {
    if (!seconds) return null;
    const m = Math.floor(seconds / 60);
    const h = Math.floor(m / 60);
    return h > 0 ? `${h}h ${m % 60}m` : `${m}m`;
  }
</script>

<div class="p-6">
  <h1 class="text-2xl font-bold mb-4">Films</h1>

  {#if error}
    <p class="text-red-600 text-sm mb-4">{error}</p>
  {/if}

  {#if loading}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if films.length === 0}
    <p class="text-sm text-muted-foreground">
      No films yet. Add a directory and scan it under <a class="underline" href="/directories">Directories</a>.
    </p>
  {:else}
    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
      {#each films as film (film.id)}
        <a href={`/films/${film.id}`} class="group">
          <Card.Root class="overflow-hidden hover:ring-2 hover:ring-primary transition">
            <div class="aspect-[2/3] bg-muted">
              {#if posterUrl(film)}
                <img
                  src={posterUrl(film)}
                  alt={film.title}
                  class="w-full h-full object-cover"
                  loading="lazy"
                />
              {/if}
            </div>
            <Card.Content class="p-3">
              <div class="font-medium truncate">{film.title}</div>
              <div class="text-xs text-muted-foreground">
                {film.release_date?.slice(0, 4) ?? '—'}
                {#if formatDuration(film.runtime ? film.runtime * 60 : null)}
                  · {formatDuration(film.runtime ? film.runtime * 60 : null)}
                {/if}
              </div>
            </Card.Content>
          </Card.Root>
        </a>
      {/each}
    </div>
  {/if}
</div>
