<script lang="ts">
  import { api, type Library, type LibraryKind, type ScanReport } from '$lib/api';
  import * as Select from '$lib/components/ui/select';
  import { FolderPlus, RefreshCw, Trash2 } from '$lib/lucide';

  let libraries: Library[] = $state([]);
  let loading = $state(true);
  let busy = $state(false);
  let kind: LibraryKind = $state('mixed');

  const kindLabels: Record<LibraryKind, string> = {
    mixed: 'Auto-detect',
    movies: 'Movies only',
    series: 'Series only',
  };

  const selectedKindLabel = $derived(kindLabels[kind]);
  let mpvOk = $state<boolean | null>(null);
  let lastReport: ScanReport | null = $state(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    try {
      [libraries, mpvOk] = await Promise.all([api.listLibraries(), api.checkMpv()]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function add() {
    error = null;
    const path = await api.pickFolder();
    if (!path) return;
    busy = true;
    try {
      await api.addLibrary(path, kind);
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove(id: number) {
    busy = true;
    try {
      await api.removeLibrary(id);
      libraries = libraries.filter((l) => l.id !== id);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function rescan() {
    busy = true;
    lastReport = null;
    try {
      lastReport = await api.scanLibraries();
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="px-6 py-8 lg:px-12">
  <header class="mb-8 flex flex-col gap-2">
    <h1 class="text-3xl font-bold tracking-tight">Libraries</h1>
    <p class="text-muted-foreground">
      Add folders that contain your movies or series. Rustflix will index them for playback.
    </p>
  </header>

  {#if mpvOk === false}
    <div
      class="mb-6 rounded-md border border-yellow-500/30 bg-yellow-500/10 px-4 py-3 text-sm text-yellow-200"
    >
      <strong>Bundled mpv is missing.</strong> Your install seems incomplete — try reinstalling
      Rustflix.
    </div>
  {/if}

  {#if error}
    <div class="mb-6 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive-foreground">
      {error}
    </div>
  {/if}

  <section class="mb-8 rounded-lg border border-border bg-card p-5">
    <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
      Add a folder
    </h2>
    <div class="flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2 text-sm">
        <span class="text-muted-foreground">Treat as:</span>
        <Select.Root
          type="single"
          value={kind}
          onValueChange={(value) => {
            kind = value as LibraryKind;
          }}
        >
          <Select.Trigger class="w-[170px]" aria-label="Library kind">
            {selectedKindLabel}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="mixed" label="Auto-detect">Auto-detect</Select.Item>
            <Select.Item value="movies" label="Movies only">Movies only</Select.Item>
            <Select.Item value="series" label="Series only">Series only</Select.Item>
          </Select.Content>
        </Select.Root>
      </div>
      <button
        type="button"
        onclick={add}
        disabled={busy}
        class="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground shadow transition hover:bg-primary/90 disabled:opacity-50"
      >
        <FolderPlus class="size-4" />
        Pick folder
      </button>
      <button
        type="button"
        onclick={rescan}
        disabled={busy || libraries.length === 0}
        class="inline-flex items-center gap-2 rounded-md bg-secondary px-4 py-2 text-sm font-semibold text-secondary-foreground transition hover:bg-accent disabled:opacity-50"
      >
        <RefreshCw class="size-4 {busy ? 'animate-spin' : ''}" />
        Rescan all
      </button>
    </div>
    {#if lastReport}
      <p class="mt-3 text-sm text-muted-foreground">
        Scanned {lastReport.libraries_scanned} libraries · added
        {lastReport.movies_added} movies, {lastReport.shows_added} shows,
        {lastReport.episodes_added} episodes.
      </p>
    {/if}
  </section>

  <section>
    <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
      Your libraries
    </h2>
    {#if loading}
      <p class="text-muted-foreground">Loading…</p>
    {:else if libraries.length === 0}
      <div class="rounded-lg border border-dashed border-border bg-card/50 p-8 text-center text-muted-foreground">
        No libraries yet. Pick a folder above to get started.
      </div>
    {:else}
      <ul class="divide-y divide-border rounded-lg border border-border bg-card">
        {#each libraries as lib (lib.id)}
          <li class="flex items-center gap-4 px-5 py-4">
            <div class="min-w-0 flex-1">
              <div class="truncate font-medium">{lib.path}</div>
              <div class="text-xs uppercase tracking-wide text-muted-foreground">
                {lib.kind}
              </div>
            </div>
            <button
              type="button"
              onclick={() => remove(lib.id)}
              disabled={busy}
              class="inline-flex size-9 items-center justify-center rounded-md text-muted-foreground transition hover:bg-destructive/20 hover:text-destructive-foreground"
              aria-label="Remove library"
            >
              <Trash2 class="size-4" />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>
