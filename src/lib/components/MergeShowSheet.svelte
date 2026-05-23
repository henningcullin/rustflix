<script lang="ts">
  import { api, type EpisodeRef, type Show } from '$lib/api';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import * as Sheet from '$lib/components/ui/sheet';
  import { AlertTriangle, GitMerge } from '$lib/lucide';

  type Props = {
    open: boolean;
    show: Show;
    onClose: () => void;
    onMerged: () => void | Promise<void>;
  };

  let { open = $bindable(), show, onClose, onMerged }: Props = $props();

  let candidates: Show[] = $state([]);
  let loading = $state(false);
  let query = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let conflicts: EpisodeRef[] = $state([]);
  let conflictWith = $state<string | null>(null);

  $effect(() => {
    if (open) {
      void loadCandidates();
      return;
    }
    candidates = [];
    query = '';
    conflicts = [];
    conflictWith = null;
    error = null;
  });

  async function loadCandidates() {
    loading = true;
    error = null;
    try {
      const all = await api.listShows();
      candidates = all.filter(
        (other) => other.id !== show.id && other.library_id === show.library_id,
      );
    } catch (caught) {
      error = String(caught);
    } finally {
      loading = false;
    }
  }

  const filtered = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    if (needle.length === 0) {
      return candidates;
    }
    return candidates.filter((candidate) => candidate.title.toLowerCase().includes(needle));
  });

  async function attemptMerge(candidate: Show) {
    busy = true;
    conflicts = [];
    conflictWith = null;
    error = null;
    try {
      const outcome = await api.mergeShows(show.id, candidate.id);
      if (outcome.conflicts.length > 0) {
        conflicts = outcome.conflicts;
        conflictWith = candidate.title;
        return;
      }
      await onMerged();
      onClose();
    } catch (caught) {
      error = String(caught);
    } finally {
      busy = false;
    }
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content side="right" class="flex w-full flex-col gap-4 sm:max-w-md">
    <Sheet.Header>
      <Sheet.Title>Merge with another show</Sheet.Title>
      <Sheet.Description>
        Pick a show in this library — its episodes will move into "{show.title}" and the other
        entry will be deleted.
      </Sheet.Description>
    </Sheet.Header>

    {#if error}
      <div
        class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive-foreground"
      >
        {error}
      </div>
    {/if}

    {#if conflicts.length > 0}
      <div class="rounded-md border border-yellow-500/30 bg-yellow-500/10 p-3 text-sm">
        <div class="mb-2 flex items-center gap-2 font-medium text-yellow-200">
          <AlertTriangle class="size-4" />
          <span>Can't merge with {conflictWith}</span>
        </div>
        <p class="mb-2 text-muted-foreground">
          These episodes exist in both shows — resolve the duplicates first, then try again:
        </p>
        <ul class="list-disc pl-5 text-muted-foreground">
          {#each conflicts as conflict (`${conflict.season}-${conflict.episode}`)}
            <li>
              S{String(conflict.season).padStart(2, '0')}E{String(conflict.episode).padStart(2, '0')}
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <Input bind:value={query} placeholder="Search shows…" />

    <div class="-mx-2 flex-1 overflow-y-auto px-2">
      {#if loading}
        <p class="text-sm text-muted-foreground">Loading…</p>
      {:else if filtered.length === 0}
        <p class="text-sm text-muted-foreground">
          {query ? 'No matching shows.' : 'No other shows in this library.'}
        </p>
      {:else}
        <ul class="divide-y divide-border rounded-md border border-border">
          {#each filtered as candidate (candidate.id)}
            <li>
              <button
                type="button"
                onclick={() => attemptMerge(candidate)}
                disabled={busy}
                class="flex w-full items-center justify-between gap-3 px-4 py-3 text-left transition-colors hover:bg-accent disabled:opacity-50"
              >
                <div class="min-w-0 flex-1">
                  <div class="truncate font-medium">{candidate.title}</div>
                  <div class="text-xs text-muted-foreground">
                    {candidate.episode_count} episodes{candidate.year ? ` · ${candidate.year}` : ''}
                  </div>
                </div>
                <GitMerge class="size-4 shrink-0 text-muted-foreground" />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <Sheet.Footer>
      <Button variant="ghost" onclick={onClose} disabled={busy}>Cancel</Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
