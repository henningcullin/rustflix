<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import { Input } from '$lib/components/ui/input/index';
  import * as Card from '$lib/components/ui/card/index';
  import * as Sheet from '$lib/components/ui/sheet/index';
  import { tmdbSearch, tmdbImportFilm } from '$lib/api/tmdb';
  import type { ScanResult, TmdbSearchResult, UnmatchedFile } from '$lib/types';

  let {
    result,
    open = $bindable(false),
    onImported,
  }: {
    result: ScanResult | null;
    open?: boolean;
    onImported?: () => void;
  } = $props();

  let searchFor = $state<UnmatchedFile | null>(null);
  let query = $state('');
  let results = $state<TmdbSearchResult[]>([]);
  let searching = $state(false);
  let importing = $state<number | null>(null);
  let error = $state<string | null>(null);

  function startSearch(file: UnmatchedFile) {
    searchFor = file;
    query = file.display_name.replace(/[._-]+/g, ' ').replace(/\b(19|20)\d{2}\b.*/, '').trim();
    results = [];
    error = null;
    void doSearch();
  }

  async function doSearch() {
    if (!query.trim()) return;
    searching = true;
    error = null;
    try {
      results = await tmdbSearch(query);
    } catch (err) {
      error = String(err);
    } finally {
      searching = false;
    }
  }

  async function pick(r: TmdbSearchResult) {
    if (!searchFor) return;
    importing = r.id;
    error = null;
    try {
      await tmdbImportFilm(searchFor.file_path, r.id);
      searchFor = null;
      results = [];
      query = '';
      onImported?.();
    } catch (err) {
      error = String(err);
    } finally {
      importing = null;
    }
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content class="sm:max-w-xl overflow-y-auto">
    <Sheet.Header>
      <Sheet.Title>Scan results</Sheet.Title>
      <Sheet.Description>
        {#if result}
          {result.matched.length} matched, {result.unmatched.length} unmatched.
        {/if}
      </Sheet.Description>
    </Sheet.Header>

    {#if searchFor}
      <div class="p-4 space-y-4">
        <p class="text-sm text-muted-foreground truncate">{searchFor.file_path}</p>
        <div class="flex gap-2">
          <Input
            bind:value={query}
            placeholder="Film title…"
            onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') void doSearch(); }}
          />
          <Button onclick={doSearch} disabled={searching}>
            {searching ? 'Searching…' : 'Search'}
          </Button>
          <Button variant="ghost" onclick={() => { searchFor = null; }}>Back</Button>
        </div>
        {#if error}
          <p class="text-red-600 text-sm">{error}</p>
        {/if}
        <div class="space-y-2">
          {#each results as r (r.id)}
            <Card.Root>
              <Card.Content class="flex gap-3 p-3">
                {#if r.poster_path}
                  <img
                    src={`https://image.tmdb.org/t/p/w92${r.poster_path}`}
                    alt=""
                    class="w-16 h-24 object-cover rounded"
                  />
                {:else}
                  <div class="w-16 h-24 bg-muted rounded"></div>
                {/if}
                <div class="flex-1 min-w-0">
                  <div class="font-medium truncate">{r.title}</div>
                  <div class="text-xs text-muted-foreground">
                    {r.release_date ?? '—'}
                    {#if r.vote_average !== null && r.vote_average !== undefined}
                      · ★ {r.vote_average.toFixed(1)}
                    {/if}
                  </div>
                  <p class="text-xs text-muted-foreground line-clamp-3 mt-1">
                    {r.overview ?? ''}
                  </p>
                </div>
                <Button
                  size="sm"
                  onclick={() => pick(r)}
                  disabled={importing !== null}
                >
                  {importing === r.id ? 'Importing…' : 'Pick'}
                </Button>
              </Card.Content>
            </Card.Root>
          {/each}
        </div>
      </div>
    {:else}
      <div class="p-4 space-y-4">
        {#if result && result.unmatched.length === 0 && result.matched.length === 0}
          <p class="text-sm text-muted-foreground">No video files found.</p>
        {/if}
        {#each result?.unmatched ?? [] as file (file.file_path)}
          <div class="flex items-center justify-between gap-3">
            <div class="min-w-0">
              <div class="font-medium truncate">{file.display_name}</div>
              <div class="text-xs text-muted-foreground truncate">{file.file_path}</div>
            </div>
            <Button size="sm" onclick={() => startSearch(file)}>Search TMDb</Button>
          </div>
        {/each}

        {#if (result?.matched?.length ?? 0) > 0}
          <h3 class="text-sm font-semibold pt-4">Already matched</h3>
          {#each result?.matched ?? [] as file (file.file_path)}
            <div class="flex items-center justify-between gap-3 opacity-70">
              <div class="min-w-0">
                <div class="font-medium truncate">{file.title}</div>
                <div class="text-xs text-muted-foreground truncate">{file.file_path}</div>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </Sheet.Content>
</Sheet.Root>
