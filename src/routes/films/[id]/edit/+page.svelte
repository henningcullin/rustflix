<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { api, pickImageFile, type MetadataPatch, type Movie } from '$lib/api';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { ChevronLeft, Image as ImageIcon, RotateCcw } from '$lib/lucide';

  let movie: Movie | null = $state(null);
  let loading = $state(true);
  let saving = $state(false);
  let busyPoster = $state(false);
  let error = $state<string | null>(null);

  let titleDraft = $state('');
  let yearDraft = $state('');
  let overviewDraft = $state('');

  const id = $derived(Number($page.params.id));

  $effect(() => {
    if (!Number.isFinite(id)) {
      return;
    }
    void load(id);
  });

  async function load(movieId: number) {
    loading = true;
    error = null;
    try {
      const loaded = await api.getMovie(movieId);
      movie = loaded;
      titleDraft = loaded.title;
      yearDraft = loaded.year == null ? '' : String(loaded.year);
      overviewDraft = loaded.overview ?? '';
    } catch (caught) {
      error = String(caught);
    } finally {
      loading = false;
    }
  }

  function buildPatch(): MetadataPatch | null {
    if (!movie) {
      return null;
    }

    const patch: MetadataPatch = {};
    const nextTitle = titleDraft.trim();
    if (nextTitle.length > 0 && nextTitle !== movie.title) {
      patch.title = nextTitle;
    }

    const trimmedYear = yearDraft.trim();
    if (trimmedYear.length > 0) {
      const parsed = Number(trimmedYear);
      if (Number.isInteger(parsed) && parsed > 0 && parsed !== movie.year) {
        patch.year = parsed;
      }
    }

    const nextOverview = overviewDraft.trim();
    if (nextOverview.length > 0 && nextOverview !== (movie.overview ?? '')) {
      patch.overview = nextOverview;
    }

    return Object.keys(patch).length === 0 ? null : patch;
  }

  async function save() {
    if (!movie) {
      return;
    }

    const patch = buildPatch();
    if (!patch) {
      await goto(`/films/${movie.id}`);
      return;
    }

    saving = true;
    error = null;
    try {
      await api.updateMovieMetadata(movie.id, patch);
      await goto(`/films/${movie.id}`);
    } catch (caught) {
      error = String(caught);
    } finally {
      saving = false;
    }
  }

  async function changePoster() {
    if (!movie) {
      return;
    }
    busyPoster = true;
    try {
      const source = await pickImageFile();
      if (!source) {
        return;
      }
      const updated = await api.setMoviePosterFromFile(movie.id, source);
      movie = updated;
    } catch (caught) {
      error = String(caught);
    } finally {
      busyPoster = false;
    }
  }

  async function resetPoster() {
    if (!movie) {
      return;
    }
    busyPoster = true;
    try {
      const updated = await api.resetMoviePoster(movie.id);
      movie = updated;
    } catch (caught) {
      error = String(caught);
    } finally {
      busyPoster = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <a
    href={movie ? `/films/${movie.id}` : '/films'}
    class="mb-6 inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
  >
    <ChevronLeft class="size-4" />
    Back
  </a>

  <header class="mb-6">
    <h1 class="text-3xl font-bold tracking-tight">Edit movie</h1>
    {#if movie}
      <p class="text-sm text-muted-foreground">{movie.title}</p>
    {/if}
  </header>

  {#if error}
    <div
      class="mb-6 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive-foreground"
    >
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-muted-foreground">Loading…</p>
  {:else if !movie}
    <p class="text-destructive-foreground">Movie not found.</p>
  {:else}
    <div class="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>Basic</CardTitle>
          <CardDescription>Title, year, and short overview.</CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-4">
          <label class="flex flex-col gap-1.5 text-sm">
            <span class="font-medium">Title</span>
            <Input bind:value={titleDraft} placeholder="Movie title" />
          </label>
          <label class="flex flex-col gap-1.5 text-sm">
            <span class="font-medium">Year</span>
            <Input
              type="text"
              bind:value={yearDraft}
              placeholder="e.g. 1999"
              inputmode="numeric"
            />
          </label>
          <label class="flex flex-col gap-1.5 text-sm">
            <span class="font-medium">Overview</span>
            <textarea
              bind:value={overviewDraft}
              rows="4"
              placeholder="Short description of the movie"
              class="rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            ></textarea>
          </label>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Artwork</CardTitle>
          <CardDescription>
            Scanner auto-discovers <code class="font-mono text-xs">poster.jpg</code> next to the
            movie file. Override here for a custom image.
          </CardDescription>
        </CardHeader>
        <CardContent class="flex flex-wrap items-start gap-5">
          <div class="aspect-[2/3] w-32 shrink-0 overflow-hidden rounded-md border border-border bg-card">
            {#if movie.poster_path}
              <img
                src={movie.poster_path}
                alt=""
                class="h-full w-full object-cover"
              />
            {:else}
              <div class="flex h-full w-full items-center justify-center bg-muted">
                <ImageIcon class="size-8 text-muted-foreground/60" />
              </div>
            {/if}
          </div>
          <div class="flex flex-col gap-2">
            <p class="text-sm">
              {#if movie.poster_origin === 'manual'}
                Currently using your uploaded poster.
              {:else if movie.poster_origin === 'auto'}
                Currently using the poster found next to your files.
              {:else}
                No poster set. Drop a <code class="font-mono text-xs">poster.jpg</code> next to the
                movie file or pick one below.
              {/if}
            </p>
            <div class="flex flex-wrap gap-2">
              <Button variant="default" size="sm" onclick={changePoster} disabled={busyPoster}>
                <ImageIcon class="mr-1.5 size-4" />
                Change poster
              </Button>
              {#if movie.poster_origin === 'manual'}
                <Button variant="secondary" size="sm" onclick={resetPoster} disabled={busyPoster}>
                  <RotateCcw class="mr-1.5 size-4" />
                  Reset to auto
                </Button>
              {/if}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Advanced</CardTitle>
          <CardDescription>
            More fields will appear here as Rustflix learns about your media — cast, ratings,
            language, genres.
          </CardDescription>
        </CardHeader>
      </Card>

      <div class="flex justify-end gap-2">
        <Button
          variant="ghost"
          onclick={() => movie && goto(`/films/${movie.id}`)}
          disabled={saving}
        >
          Cancel
        </Button>
        <Button onclick={save} disabled={saving}>
          {saving ? 'Saving…' : 'Save'}
        </Button>
      </div>
    </div>
  {/if}
</div>
