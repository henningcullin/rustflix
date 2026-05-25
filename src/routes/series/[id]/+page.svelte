<script lang="ts">
  import { page } from '$app/stores';
  import {
    api,
    formatRuntime,
    pickImageFile,
    progressPct,
    type Season,
    type Show,
  } from '$lib/api';
  import EpisodeTitleEditor from '$lib/components/EpisodeTitleEditor.svelte';
  import HeroBanner from '$lib/components/HeroBanner.svelte';
  import MergeShowSheet from '$lib/components/MergeShowSheet.svelte';
  import { Check, Circle, GitMerge, Pencil, Play } from '$lib/lucide';

  let show: Show | null = $state(null);
  let seasons: Season[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedSeason = $state<number | null>(null);
  let mergeOpen = $state(false);

  const id = $derived(Number($page.params.id));

  $effect(() => {
    if (!Number.isFinite(id)) {
      return;
    }
    void load(id);
  });

  async function load(showId: number) {
    loading = true;
    error = null;
    try {
      [show, seasons] = await Promise.all([api.getShow(showId), api.getSeasons(showId)]);
      if (selectedSeason === null && seasons.length > 0) {
        selectedSeason = seasons[0].season;
      }
    } catch (caught) {
      error = String(caught);
    } finally {
      loading = false;
    }
  }

  const nextUp = $derived.by(() => {
    for (const season of seasons) {
      for (const episode of season.episodes) {
        if (!episode.watched) {
          return episode;
        }
      }
    }
    return null;
  });

  async function playEpisode(epId: number, fromStart = false) {
    try {
      await api.playEpisode(epId, fromStart ? 0 : undefined);
      if (show) {
        await load(show.id);
      }
    } catch (caught) {
      error = String(caught);
    }
  }

  async function saveTitle(next: string) {
    if (!show) {
      return;
    }
    try {
      const updated = await api.updateShowMetadata(show.id, { title: next });
      show = { ...show, title: updated.title };
    } catch (caught) {
      error = String(caught);
    }
  }

  async function changePoster() {
    if (!show) {
      return;
    }
    try {
      const source = await pickImageFile();
      if (!source) {
        return;
      }
      const updated = await api.setShowPosterFromFile(show.id, source);
      show = {
        ...show,
        poster_path: updated.poster_path,
        poster_origin: updated.poster_origin,
      };
    } catch (caught) {
      error = String(caught);
    }
  }

  async function resetPoster() {
    if (!show) {
      return;
    }
    try {
      const updated = await api.resetShowPoster(show.id);
      show = {
        ...show,
        poster_path: updated.poster_path,
        poster_origin: updated.poster_origin,
      };
    } catch (caught) {
      error = String(caught);
    }
  }

  async function saveEpisodeTitle(episodeId: number, nextTitle: string) {
    try {
      const updated = await api.updateEpisodeTitle(episodeId, nextTitle);

      for (const season of seasons) {
        const episode = season.episodes.find((candidate) => candidate.id === episodeId);
        if (!episode) {
          continue;
        }

        episode.title = updated.title;
        return;
      }
    } catch (caught) {
      error = String(caught);
    }
  }

  async function toggleEpisode(episodeId: number, watched: boolean) {
    const next = !watched;
    try {
      await api.setWatched('episode', episodeId, next);

      for (const season of seasons) {
        const episode = season.episodes.find((candidate) => candidate.id === episodeId);
        if (!episode) {
          continue;
        }

        episode.watched = next;
        if (show) {
          show.watched_count += next ? 1 : -1;
        }
        return;
      }
    } catch (caught) {
      error = String(caught);
    }
  }

  const activeSeason = $derived(
    seasons.find((season) => season.season === selectedSeason) ?? seasons[0] ?? null,
  );
</script>

{#if loading}
  <div class="flex h-[60vh] items-center justify-center text-muted-foreground">Loading…</div>
{:else if error || !show}
  <div class="px-6 py-12 text-destructive-foreground">{error ?? 'Show not found.'}</div>
{:else}
  <HeroBanner
    title={show.title}
    subtitle={`Series · ${show.watched_count}/${show.episode_count} watched`}
    overview={show.overview ?? null}
    href={`/series/${show.id}`}
    year={show.year}
    backdrop={show.poster_path ?? null}
    posterIsManual={show.poster_origin === 'manual'}
    onTitleSave={saveTitle}
    onPosterChange={changePoster}
    onPosterReset={resetPoster}
  />

  <div class="px-6 py-8 lg:px-12">
    <div class="mb-8 flex flex-wrap items-center gap-3">
      {#if nextUp}
        <button
          type="button"
          onclick={() => playEpisode(nextUp.id, nextUp.progress_seconds < 30)}
          class="inline-flex items-center gap-2 rounded-md bg-primary px-5 py-2.5 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/30 transition hover:bg-primary/90"
        >
          <Play class="size-4 fill-current" />
          {nextUp.progress_seconds > 30 ? 'Resume' : 'Play'} S{String(nextUp.season).padStart(2, '0')}E{String(nextUp.episode).padStart(2, '0')}
        </button>
        <span class="text-sm text-muted-foreground">{nextUp.title}</span>
      {/if}
      <div class="ml-auto flex items-center gap-2">
        <button
          type="button"
          onclick={() => (mergeOpen = true)}
          class="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-1.5 text-sm font-medium text-muted-foreground transition hover:bg-accent hover:text-foreground"
        >
          <GitMerge class="size-3.5" />
          Merge…
        </button>
        <a
          href={`/series/${show.id}/edit`}
          class="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-1.5 text-sm font-medium text-muted-foreground transition hover:bg-accent hover:text-foreground"
        >
          <Pencil class="size-3.5" />
          Edit details
        </a>
      </div>
    </div>


    {#if seasons.length > 1}
      <div class="mb-4 flex flex-wrap gap-2">
        {#each seasons as season (season.season)}
          {@const isActive = season.season === activeSeason?.season}
          <button
            type="button"
            onclick={() => (selectedSeason = season.season)}
            class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors {isActive
              ? 'bg-primary text-primary-foreground'
              : 'bg-secondary text-secondary-foreground hover:bg-accent'}"
          >
            Season {season.season}
          </button>
        {/each}
      </div>
    {/if}

    <MergeShowSheet
      bind:open={mergeOpen}
      {show}
      onClose={() => (mergeOpen = false)}
      onMerged={() => {
        if (show) {
          void load(show.id);
        }
      }}
    />

    {#if activeSeason}
      <ul class="divide-y divide-border overflow-hidden rounded-lg border border-border bg-card">
        {#each activeSeason.episodes as ep (ep.id)}
          <li class="group flex items-center gap-4 px-5 py-4 transition-colors hover:bg-accent/30">
            <div class="w-12 shrink-0 text-2xl font-bold text-muted-foreground">
              {String(ep.episode).padStart(2, '0')}
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <div class="min-w-0 flex-1">
                  <EpisodeTitleEditor
                    title={ep.title}
                    onSave={(next) => saveEpisodeTitle(ep.id, next)}
                  />
                </div>
                {#if ep.watched}
                  <Check class="size-4 shrink-0 text-emerald-400" />
                {/if}
              </div>
              {#if ep.progress_seconds > 0 && !ep.watched && ep.duration_seconds}
                <div class="mt-2 max-w-xs">
                  <div class="mb-1 text-xs text-muted-foreground">
                    {formatRuntime(ep.progress_seconds)} / {formatRuntime(ep.duration_seconds)}
                  </div>
                  <div class="h-1 overflow-hidden rounded-full bg-muted">
                    <div class="h-full bg-primary" style="width: {progressPct(ep)}%"></div>
                  </div>
                </div>
              {/if}
            </div>
            <div class="flex items-center gap-1 opacity-70 transition-opacity group-hover:opacity-100">
              <button
                type="button"
                onclick={() => playEpisode(ep.id)}
                class="inline-flex size-9 items-center justify-center rounded-full bg-primary text-primary-foreground shadow transition hover:bg-primary/90"
                aria-label="Play episode"
              >
                <Play class="size-4 fill-current" />
              </button>
              <button
                type="button"
                onclick={() => toggleEpisode(ep.id, ep.watched)}
                class="inline-flex size-9 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-foreground"
                aria-label={ep.watched ? 'Mark unwatched' : 'Mark watched'}
              >
                {#if ep.watched}
                  <Check class="size-4 text-emerald-400" />
                {:else}
                  <Circle class="size-4" />
                {/if}
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}
