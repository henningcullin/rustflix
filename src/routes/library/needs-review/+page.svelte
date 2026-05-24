<script lang="ts">
  import { api, type NeedsReviewItem } from '$lib/api';
  import MetadataMatchSheet from '$lib/components/MetadataMatchSheet.svelte';
  import { Button } from '$lib/components/ui/button';

  let items = $state<NeedsReviewItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let active = $state<NeedsReviewItem | null>(null);
  let sheetOpen = $state(false);

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    try {
      items = await api.listNeedsReview();
    } catch (caught) {
      error = String(caught);
    } finally {
      loading = false;
    }
  }

  function openSheet(item: NeedsReviewItem) {
    active = item;
    sheetOpen = true;
  }
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <h1 class="mb-6 text-3xl font-bold tracking-tight">Needs review</h1>
  <p class="mb-6 text-sm text-muted-foreground">
    These items couldn't be auto-linked to a TMDB record. Pick the right match.
  </p>

  {#if error}
    <p class="mb-4 text-sm text-destructive-foreground">{error}</p>
  {/if}

  {#if loading}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if items.length === 0}
    <p class="text-sm text-muted-foreground">Everything is matched. Nothing to review.</p>
  {:else}
    <ul class="flex flex-col gap-2">
      {#each items as item (item.kind + ':' + item.id)}
        <li
          class="flex items-center justify-between rounded-md border border-border bg-card px-4 py-3"
        >
          <div>
            <div class="font-medium">{item.title}</div>
            <div class="text-xs uppercase tracking-wide text-muted-foreground">
              {item.kind}{item.year ? ` · ${item.year}` : ''}
            </div>
          </div>
          <Button onclick={() => openSheet(item)}>Match…</Button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<MetadataMatchSheet
  bind:open={sheetOpen}
  item={active}
  onClose={() => (sheetOpen = false)}
  onLinked={() => load()}
/>
