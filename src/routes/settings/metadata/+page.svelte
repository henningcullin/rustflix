<script lang="ts">
  import { api, type MetadataStatusCounts } from '$lib/api';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';

  let keyDraft = $state('');
  let savedKey = $state<string | null>(null);
  let saving = $state(false);
  let counts = $state<MetadataStatusCounts | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void load();
  });

  async function load() {
    try {
      const [keyResult, countsResult] = await Promise.all([
        api.getTmdbApiKey(),
        api.metadataStatusCounts(),
      ]);
      savedKey = keyResult;
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
      await api.setTmdbApiKey(keyDraft);
      const trimmed = keyDraft.trim();
      savedKey = trimmed.length === 0 ? null : trimmed;
    } catch (caught) {
      error = String(caught);
    } finally {
      saving = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <header class="mb-6">
    <h1 class="text-3xl font-bold tracking-tight">Metadata</h1>
    <p class="text-sm text-muted-foreground">
      Rustflix can fetch posters, overviews, genres, ratings, and cast from TMDB.
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
        <CardTitle>TMDB API key</CardTitle>
        <CardDescription>
          Sign up at <a class="underline" href="https://www.themoviedb.org/settings/api">themoviedb.org</a>
          and paste your v3 API key here. Without a key, metadata sync is paused.
        </CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <Input
          bind:value={keyDraft}
          placeholder="Paste your TMDB v3 API key"
          type="password"
        />
        <div class="flex items-center gap-3">
          <Button onclick={saveKey} disabled={saving}>
            {saving ? 'Saving…' : savedKey ? 'Update key' : 'Save key'}
          </Button>
          {#if savedKey}
            <span class="text-xs text-muted-foreground">
              A key is currently stored.
            </span>
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
          <ul class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-5">
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-xs uppercase tracking-wide text-muted-foreground">Pending</div>
              <div class="text-lg font-semibold">{counts.pending}</div>
            </li>
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-xs uppercase tracking-wide text-muted-foreground">Failed</div>
              <div class="text-lg font-semibold">{counts.failed}</div>
            </li>
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-xs uppercase tracking-wide text-muted-foreground">Auth-paused</div>
              <div class="text-lg font-semibold">{counts.auth_required}</div>
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
  </div>
</div>
