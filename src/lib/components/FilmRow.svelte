<script lang="ts">
  import type { FilmListItem } from '$lib/types';

  let { title, films, emptyHint }: {
    title: string;
    films: FilmListItem[];
    emptyHint?: string;
  } = $props();

  function posterUrl(film: FilmListItem): string | null {
    if (!film.poster_path) return null;
    if (film.poster_path.startsWith('/')) {
      return `https://image.tmdb.org/t/p/w500${film.poster_path}`;
    }
    return null;
  }

  function progressPct(film: FilmListItem): number {
    if (!film.runtime || film.runtime <= 0) return 0;
    const total = film.runtime * 60;
    return Math.min(100, Math.round((film.left_off_point / total) * 100));
  }
</script>

<section class="space-y-3">
  <h2 class="text-lg font-semibold">{title}</h2>
  {#if films.length === 0}
    {#if emptyHint}
      <p class="text-sm text-muted-foreground">{emptyHint}</p>
    {/if}
  {:else}
    <div class="flex gap-4 overflow-x-auto pb-2 -mx-1 px-1 snap-x">
      {#each films as film (film.id)}
        <a
          href={`/films/${film.id}`}
          class="flex-none w-36 sm:w-44 snap-start group"
        >
          <div class="relative aspect-[2/3] bg-muted rounded-md overflow-hidden ring-1 ring-border group-hover:ring-primary transition">
            {#if posterUrl(film)}
              <img
                src={posterUrl(film)}
                alt={film.title}
                class="w-full h-full object-cover"
                loading="lazy"
              />
            {/if}
            {#if film.left_off_point > 15}
              <div class="absolute bottom-0 inset-x-0 h-1 bg-black/40">
                <div class="h-full bg-primary" style:width={`${progressPct(film)}%`}></div>
              </div>
            {/if}
          </div>
          <div class="mt-2 text-sm font-medium truncate">{film.title}</div>
          <div class="text-xs text-muted-foreground">
            {film.release_date?.slice(0, 4) ?? ''}
          </div>
        </a>
      {/each}
    </div>
  {/if}
</section>
