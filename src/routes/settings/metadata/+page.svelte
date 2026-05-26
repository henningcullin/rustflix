<script lang="ts">
  import { api, type MetadataStatusCounts } from '$lib/api';
  import { getSetting, setSetting, type MetadataMode } from '$lib/settings';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import * as Select from '$lib/components/ui/select';

  let mode = $state<MetadataMode>('prefer_tmdb');
  let keyDraft = $state('');
  let savedKey = $state<string | null>(null);
  let saving = $state(false);
  let savingMode = $state(false);
  let counts = $state<MetadataStatusCounts | null>(null);
  let error = $state<string | null>(null);

  const MODE_LABELS: Record<MetadataMode, string> = {
    off: 'Off (no metadata sync)',
    tmdb_only: 'TMDB only',
    imdb_only: 'IMDB only',
    prefer_tmdb: 'Prefer TMDB, fall back to IMDB',
    prefer_imdb: 'Prefer IMDB, fall back to TMDB',
  };

  $effect(() => {
    void load();
  });

  async function load() {
    try {
      const [keyResult, modeResult, countsResult] = await Promise.all([
        getSetting('tmdb_api_key'),
        getSetting('metadata_mode'),
        api.metadataStatusCounts(),
      ]);
      savedKey = keyResult;
      mode = modeResult;
      counts = countsResult;
      keyDraft = savedKey ?? '';
    } catch (caught) {
      error = String(caught);
    }
  }

  async function saveKey() {
    saving = true;
    error = null;
    try {
      const trimmed = keyDraft.trim();
      await setSetting('tmdb_api_key', trimmed.length === 0 ? null : trimmed);
      savedKey = trimmed.length === 0 ? null : trimmed;
    } catch (caught) {
      error = String(caught);
    } finally {
      saving = false;
    }
  }

  async function saveMode(next: MetadataMode) {
    savingMode = true;
    error = null;
    try {
      await setSetting('metadata_mode', next);
      mode = next;
    } catch (caught) {
      error = String(caught);
    } finally {
      savingMode = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <header class="mb-6">
    <h1 class="text-3xl font-bold tracking-tight">Metadata</h1>
    <p class="text-sm text-muted-foreground">
      Rustflix can fetch posters, overviews, genres, ratings, and cast from TMDB
      and IMDB.
    </p>
  </header>

  {#if error}
    <div
      class="mb-6 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive-foreground"
    >
      {error}
    </div>
  {/if}

  <div class="flex flex-col gap-6">
    <Card>
      <CardHeader>
        <CardTitle>Sync mode</CardTitle>
        <CardDescription>
          Pick which providers to use and in what order.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Select.Root
          type="single"
          value={mode}
          onValueChange={(next) => {
            if (next) {
              void saveMode(next as MetadataMode);
            }
          }}
        >
          <Select.Trigger class="w-full sm:w-[420px]" aria-label="Metadata sync mode">
            {MODE_LABELS[mode]}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="off" label={MODE_LABELS.off}>
              {MODE_LABELS.off}
            </Select.Item>
            <Select.Item value="tmdb_only" label={MODE_LABELS.tmdb_only}>
              {MODE_LABELS.tmdb_only}
            </Select.Item>
            <Select.Item value="imdb_only" label={MODE_LABELS.imdb_only}>
              {MODE_LABELS.imdb_only}
            </Select.Item>
            <Select.Item value="prefer_tmdb" label={MODE_LABELS.prefer_tmdb}>
              {MODE_LABELS.prefer_tmdb}
            </Select.Item>
            <Select.Item value="prefer_imdb" label={MODE_LABELS.prefer_imdb}>
              {MODE_LABELS.prefer_imdb}
            </Select.Item>
          </Select.Content>
        </Select.Root>
        {#if savingMode}
          <p class="mt-2 text-xs text-muted-foreground">Saving…</p>
        {/if}
      </CardContent>
    </Card>

    {#if mode !== 'off'}
      <Card>
        <CardHeader>
          <CardTitle>TMDB API key</CardTitle>
          <CardDescription>
            Sign up at <a class="underline" href="https://www.themoviedb.org/settings/api">themoviedb.org</a>
            and paste your v3 API key here.
            {#if mode === 'imdb_only'}
              <span class="block mt-1 text-xs">
                Not used while IMDB-only mode is active.
              </span>
            {/if}
          </CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-3">
          <Input
            bind:value={keyDraft}
            placeholder="Paste your TMDB v3 API key"
            type="password"
            disabled={mode === 'imdb_only'}
          />
          <div class="flex items-center gap-3">
            <Button onclick={saveKey} disabled={saving || mode === 'imdb_only'}>
              {saving ? 'Saving…' : savedKey ? 'Update key' : 'Save key'}
            </Button>
            {#if savedKey}
              <span class="text-xs text-muted-foreground">A key is currently stored.</span>
            {/if}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Sync status</CardTitle>
        </CardHeader>
        <CardContent>
          {#if counts}
            <ul class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-3">
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Pending</div>
                <div class="text-lg font-semibold">{counts.pending}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Failed</div>
                <div class="text-lg font-semibold">{counts.failed}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">TMDB auth needed</div>
                <div class="text-lg font-semibold">{counts.tmdb_auth_required}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">No provider</div>
                <div class="text-lg font-semibold">{counts.no_provider_available}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Dead-letter</div>
                <div class="text-lg font-semibold">{counts.dead_letter}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Needs review</div>
                <div class="text-lg font-semibold">{counts.needs_review}</div>
              </li>
            </ul>
          {:else}
            <p class="text-sm text-muted-foreground">Loading…</p>
          {/if}
        </CardContent>
      </Card>

      <p class="text-xs text-muted-foreground">
        Metadata powered by <a class="underline" href="https://www.themoviedb.org">TMDB</a>.
      </p>
    {:else}
      <div
        class="rounded-md border border-border bg-card px-4 py-3 text-sm text-muted-foreground"
      >
        Metadata sync is disabled. Pick a mode above to enable.
      </div>
    {/if}
  </div>
</div>
