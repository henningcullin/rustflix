<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { api, pickImageFile, type MetadataPatch, type Show } from '$lib/api';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { ChevronLeft, Image as ImageIcon, RotateCcw, Trash2 } from '$lib/lucide';

  let show: Show | null = $state(null);
  let loading = $state(true);
  let saving = $state(false);
  let busyPoster = $state(false);
  let deleting = $state(false);
  let confirmDeleteOpen = $state(false);
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

  async function load(showId: number) {
    loading = true;
    error = null;
    try {
      const loaded = await api.getShow(showId);
      show = loaded;
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
    if (!show) {
      return null;
    }

    const patch: MetadataPatch = {};
    const nextTitle = titleDraft.trim();
    if (nextTitle.length > 0 && nextTitle !== show.title) {
      patch.title = nextTitle;
    }

    const trimmedYear = yearDraft.trim();
    if (trimmedYear.length > 0) {
      const parsed = Number(trimmedYear);
      if (Number.isInteger(parsed) && parsed > 0 && parsed !== show.year) {
        patch.year = parsed;
      }
    }

    const nextOverview = overviewDraft.trim();
    if (nextOverview.length > 0 && nextOverview !== (show.overview ?? '')) {
      patch.overview = nextOverview;
    }

    return Object.keys(patch).length === 0 ? null : patch;
  }

  async function save() {
    if (!show) {
      return;
    }

    const patch = buildPatch();
    if (!patch) {
      await goto(`/series/${show.id}`);
      return;
    }

    saving = true;
    error = null;
    try {
      await api.updateShowMetadata(show.id, patch);
      await goto(`/series/${show.id}`);
    } catch (caught) {
      error = String(caught);
    } finally {
      saving = false;
    }
  }

  async function changePoster() {
    if (!show) {
      return;
    }
    busyPoster = true;
    try {
      const source = await pickImageFile();
      if (!source) {
        return;
      }
      const updated = await api.setShowPosterFromFile(show.id, source);
      show = updated;
    } catch (caught) {
      error = String(caught);
    } finally {
      busyPoster = false;
    }
  }

  async function resetPoster() {
    if (!show) {
      return;
    }
    busyPoster = true;
    try {
      const updated = await api.resetShowPoster(show.id);
      show = updated;
    } catch (caught) {
      error = String(caught);
    } finally {
      busyPoster = false;
    }
  }

  async function deleteSeries() {
    if (!show) {
      return;
    }

    deleting = true;
    error = null;
    try {
      await api.deleteShow(show.id);
      confirmDeleteOpen = false;
      await goto('/series');
    } catch (caught) {
      error = String(caught);
    } finally {
      deleting = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <a
    href={show ? `/series/${show.id}` : '/series'}
    class="mb-6 inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
  >
    <ChevronLeft class="size-4" />
    Back
  </a>

  <header class="mb-6">
    <h1 class="text-3xl font-bold tracking-tight">Edit series</h1>
    {#if show}
      <p class="text-sm text-muted-foreground">{show.title}</p>
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
  {:else if !show}
    <p class="text-destructive-foreground">Series not found.</p>
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
            <Input bind:value={titleDraft} placeholder="Series title" />
          </label>
          <label class="flex flex-col gap-1.5 text-sm">
            <span class="font-medium">Year</span>
            <Input
              type="text"
              bind:value={yearDraft}
              placeholder="e.g. 2008"
              inputmode="numeric"
            />
          </label>
          <label class="flex flex-col gap-1.5 text-sm">
            <span class="font-medium">Overview</span>
            <textarea
              bind:value={overviewDraft}
              rows="4"
              placeholder="Short description of the series"
              class="rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            ></textarea>
          </label>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Artwork</CardTitle>
          <CardDescription>
            Scanner auto-discovers <code class="font-mono text-xs">poster.jpg</code> next to your
            media. Override here for a custom image.
          </CardDescription>
        </CardHeader>
        <CardContent class="flex flex-wrap items-start gap-5">
          <div class="aspect-[2/3] w-32 shrink-0 overflow-hidden rounded-md border border-border bg-card">
            {#if show.poster_path}
              <img
                src={show.poster_path}
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
              {#if show.poster_origin === 'manual'}
                Currently using your uploaded poster.
              {:else if show.poster_origin === 'auto'}
                Currently using the poster found next to your files.
              {:else}
                No poster set. Drop a <code class="font-mono text-xs">poster.jpg</code> next to your
                media or pick one below.
              {/if}
            </p>
            <div class="flex flex-wrap gap-2">
              <Button variant="default" size="sm" onclick={changePoster} disabled={busyPoster}>
                <ImageIcon class="mr-1.5 size-4" />
                Change poster
              </Button>
              {#if show.poster_origin === 'manual'}
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

      <Card class="border-destructive/30">
        <CardHeader>
          <CardTitle>Danger zone</CardTitle>
          <CardDescription>
            Remove this series from your library. Files on disk are not deleted.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <AlertDialog.Root bind:open={confirmDeleteOpen}>
            <AlertDialog.Trigger
              class="inline-flex h-9 items-center justify-center gap-2 whitespace-nowrap rounded-md bg-destructive px-4 py-2 text-sm font-medium text-destructive-foreground shadow-sm transition-colors hover:bg-destructive/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
            >
              <Trash2 class="size-4" />
              Delete series
            </AlertDialog.Trigger>
            <AlertDialog.Content>
              <AlertDialog.Header>
                <AlertDialog.Title>Delete this series?</AlertDialog.Title>
                <AlertDialog.Description>
                  {show.title} and its {show.episode_count} episode {show.episode_count === 1
                    ? 'entry'
                    : 'entries'} will be removed from the library. The files on your disk are not
                  touched.
                </AlertDialog.Description>
              </AlertDialog.Header>
              <AlertDialog.Footer>
                <AlertDialog.Cancel disabled={deleting}>Cancel</AlertDialog.Cancel>
                <AlertDialog.Action
                  variant="destructive"
                  onclick={deleteSeries}
                  disabled={deleting}
                >
                  {deleting ? 'Deleting…' : 'Delete'}
                </AlertDialog.Action>
              </AlertDialog.Footer>
            </AlertDialog.Content>
          </AlertDialog.Root>
        </CardContent>
      </Card>

      <div class="flex justify-end gap-2">
        <Button
          variant="ghost"
          onclick={() => show && goto(`/series/${show.id}`)}
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
