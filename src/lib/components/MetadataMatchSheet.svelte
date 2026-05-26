<script lang="ts">
  import {
    api,
    type MatchCandidate,
    type NeedsReviewItem,
  } from '$lib/api';
  import { getSetting } from '$lib/settings';
  import { Button } from '$lib/components/ui/button';
  import * as Sheet from '$lib/components/ui/sheet';

  type Props = {
    open: boolean;
    item: NeedsReviewItem | null;
    onClose: () => void;
    onLinked: () => void;
  };

  let { open = $bindable(), item, onClose, onLinked }: Props = $props();

  let activeProvider = $state<'tmdb' | 'imdb'>('tmdb');
  let candidates = $state<MatchCandidate[]>([]);
  let searching = $state(false);
  let error = $state<string | null>(null);
  let hasTmdbKey = $state(false);

  $effect(() => {
    if (open && item) {
      void initialise();
    }
  });

  async function initialise() {
    try {
      const [mode, key] = await Promise.all([
        getSetting('metadata_mode'),
        getSetting('tmdb_api_key'),
      ]);
      hasTmdbKey = key !== null && key.length > 0;
      activeProvider = preferredProviderForMode(mode, hasTmdbKey);
      await runSearch();
    } catch (caught) {
      error = String(caught);
    }
  }

  function preferredProviderForMode(
    mode: string,
    hasKey: boolean,
  ): 'tmdb' | 'imdb' {
    if (mode === 'imdb_only' || mode === 'prefer_imdb') {
      return 'imdb';
    }

    if (mode === 'tmdb_only' || mode === 'prefer_tmdb') {
      if (hasKey) {
        return 'tmdb';
      }
      return 'imdb';
    }

    return 'tmdb';
  }

  async function runSearch() {
    if (!item) {
      return;
    }

    searching = true;
    error = null;
    candidates = [];

    try {
      candidates = await api.metadataSearch(
        item.kind,
        item.title,
        item.year,
        activeProvider,
      );
    } catch (caught) {
      error = String(caught);
    } finally {
      searching = false;
    }
  }

  async function selectProvider(next: 'tmdb' | 'imdb') {
    if (next === 'tmdb' && !hasTmdbKey) {
      return;
    }

    activeProvider = next;
    await runSearch();
  }

  async function pick(candidate: MatchCandidate) {
    if (!item) {
      return;
    }

    error = null;

    try {
      await api.linkMetadata(
        item.kind,
        item.id,
        activeProvider,
        candidate.provider_id,
      );
      onLinked();
      onClose();
    } catch (caught) {
      error = String(caught);
    }
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content side="right" class="w-full sm:max-w-md">
    <Sheet.Header>
      <Sheet.Title>Find a match</Sheet.Title>
      <Sheet.Description>
        {item ? `${item.title}${item.year ? ` (${item.year})` : ''}` : ''}
      </Sheet.Description>
    </Sheet.Header>

    <div class="mt-4 flex gap-1 rounded-md border border-border bg-card p-1">
      <button
        type="button"
        disabled={!hasTmdbKey}
        onclick={() => selectProvider('tmdb')}
        title={!hasTmdbKey ? 'Add a TMDB key under Settings → Metadata' : undefined}
        class="flex-1 rounded px-3 py-1.5 text-sm transition-colors disabled:opacity-50 {activeProvider === 'tmdb' ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'}"
      >
        TMDB
      </button>
      <button
        type="button"
        onclick={() => selectProvider('imdb')}
        class="flex-1 rounded px-3 py-1.5 text-sm transition-colors {activeProvider === 'imdb' ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'}"
      >
        IMDB
      </button>
    </div>

    {#if error}
      <p class="mt-3 text-sm text-destructive-foreground">{error}</p>
    {/if}

    {#if searching}
      <p class="mt-3 text-sm text-muted-foreground">Searching…</p>
    {:else}
      <ul class="mt-4 flex flex-col gap-2">
        {#each candidates as candidate (candidate.provider + ':' + candidate.provider_id)}
          <li>
            <button
              type="button"
              onclick={() => pick(candidate)}
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-left text-sm transition-colors hover:bg-accent"
            >
              <div class="font-medium">{candidate.title}</div>
              <div class="text-xs text-muted-foreground">
                {candidate.year ?? '—'} · {candidate.provider.toUpperCase()} · {candidate.provider_id}
              </div>
            </button>
          </li>
        {/each}
        {#if candidates.length === 0 && !searching}
          <li class="text-sm text-muted-foreground">No candidates found.</li>
        {/if}
      </ul>
    {/if}

    <Sheet.Footer class="mt-6">
      <Button variant="ghost" onclick={onClose}>Close</Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
