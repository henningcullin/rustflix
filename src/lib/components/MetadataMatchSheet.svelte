<script lang="ts">
  import {
    api,
    type MatchCandidate,
    type NeedsReviewItem,
  } from '$lib/api';
  import { Button } from '$lib/components/ui/button';
  import * as Sheet from '$lib/components/ui/sheet';

  type Props = {
    open: boolean;
    item: NeedsReviewItem | null;
    onClose: () => void;
    onLinked: () => void;
  };

  let { open = $bindable(), item, onClose, onLinked }: Props = $props();

  let candidates = $state<MatchCandidate[]>([]);
  let searching = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (open && item) {
      void runSearch();
    }
  });

  async function runSearch() {
    if (!item) {
      return;
    }
    searching = true;
    error = null;
    candidates = [];
    try {
      candidates = await api.metadataSearch(item.kind, item.title, item.year);
    } catch (caught) {
      error = String(caught);
    } finally {
      searching = false;
    }
  }

  async function pick(candidate: MatchCandidate) {
    if (!item) {
      return;
    }
    error = null;
    try {
      await api.linkMetadata(item.kind, item.id, candidate.provider_id);
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

    {#if error}
      <p class="mt-3 text-sm text-destructive-foreground">{error}</p>
    {/if}

    {#if searching}
      <p class="mt-3 text-sm text-muted-foreground">Searching…</p>
    {:else}
      <ul class="mt-4 flex flex-col gap-2">
        {#each candidates as candidate (candidate.provider_id)}
          <li>
            <button
              type="button"
              onclick={() => pick(candidate)}
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-left text-sm transition-colors hover:bg-accent"
            >
              <div class="font-medium">{candidate.title}</div>
              {#if candidate.year}
                <div class="text-xs text-muted-foreground">{candidate.year}</div>
              {/if}
            </button>
          </li>
        {/each}
        {#if candidates.length === 0}
          <li class="text-sm text-muted-foreground">No candidates found.</li>
        {/if}
      </ul>
    {/if}

    <Sheet.Footer class="mt-6">
      <Button variant="ghost" onclick={onClose}>Close</Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
