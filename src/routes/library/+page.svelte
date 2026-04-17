<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import * as Card from '$lib/components/ui/card/index';
  import { Plus, RefreshCw, Trash2, CircleAlert } from '$lib/lucide';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    addDirectory,
    deleteDirectory,
    listDirectories,
    scanDirectory,
  } from '$lib/api/directories';
  import ScanResults from '$lib/components/ScanResults.svelte';
  import type { Directory, ScanResult } from '$lib/types';
  import { onMount } from 'svelte';

  let directories = $state<Directory[]>([]);
  let loading = $state(false);
  let working = $state<number | null>(null);
  let adding = $state(false);
  let scanResult = $state<ScanResult | null>(null);
  let sheetOpen = $state(false);
  let error = $state<string | null>(null);

  onMount(refresh);

  async function refresh() {
    loading = true;
    error = null;
    try {
      directories = await listDirectories();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function pickAndAdd() {
    adding = true;
    error = null;
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string') {
        const dir = await addDirectory(selected);
        await refresh();
        scanResult = await scanDirectory(dir.id);
        if (scanResult.unmatched.length > 0) sheetOpen = true;
      }
    } catch (err) {
      error = String(err);
    } finally {
      adding = false;
    }
  }

  async function rescan(dir: Directory) {
    working = dir.id;
    error = null;
    try {
      scanResult = await scanDirectory(dir.id);
      sheetOpen = true;
    } catch (err) {
      error = String(err);
    } finally {
      working = null;
    }
  }

  async function remove(dir: Directory) {
    working = dir.id;
    try {
      await deleteDirectory(dir.id);
      directories = directories.filter((d) => d.id !== dir.id);
    } catch (err) {
      error = String(err);
    } finally {
      working = null;
    }
  }
</script>

<div class="p-6 max-w-4xl mx-auto">
  <div class="flex items-center justify-between mb-1">
    <h1 class="text-2xl font-bold">Library</h1>
    <Button onclick={pickAndAdd} disabled={adding}>
      <Plus class="size-4" /> {adding ? 'Adding…' : 'Add folder'}
    </Button>
  </div>
  <p class="text-sm text-muted-foreground mb-6">
    Folders where your films live. We'll scan them for you.
  </p>

  {#if error}
    <p class="text-red-600 text-sm mb-4">{error}</p>
  {/if}

  {#if scanResult && scanResult.unmatched.length > 0 && !sheetOpen}
    <button
      class="w-full flex items-center gap-3 rounded-md border bg-accent/30 p-3 mb-4 text-left hover:bg-accent/50 transition"
      onclick={() => (sheetOpen = true)}
    >
      <CircleAlert class="size-5 text-amber-600" />
      <div class="flex-1">
        <div class="text-sm font-medium">
          {scanResult.unmatched.length} unmatched{' '}{scanResult.unmatched.length === 1 ? 'film' : 'films'}
        </div>
        <div class="text-xs text-muted-foreground">
          Review suggestions to pair them with TMDb info.
        </div>
      </div>
    </button>
  {/if}

  {#if loading}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if directories.length === 0}
    <Card.Root class="text-center py-10">
      <Card.Content>
        <p class="text-sm text-muted-foreground">
          No folders yet. Click <em>Add folder</em> to pick one.
        </p>
      </Card.Content>
    </Card.Root>
  {:else}
    <div class="space-y-3">
      {#each directories as dir (dir.id)}
        <Card.Root>
          <Card.Content class="flex items-center justify-between gap-3 p-4">
            <div class="min-w-0">
              <div class="font-medium truncate">{dir.path}</div>
              <div class="text-xs text-muted-foreground">
                {dir.recursive ? 'Including subfolders' : 'Top level only'}
              </div>
            </div>
            <div class="flex gap-1">
              <Button
                variant="ghost"
                size="sm"
                onclick={() => rescan(dir)}
                disabled={working === dir.id}
                title="Rescan"
              >
                <RefreshCw class={working === dir.id ? 'size-4 animate-spin' : 'size-4'} />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onclick={() => remove(dir)}
                disabled={working === dir.id}
                title="Remove"
              >
                <Trash2 class="size-4" />
              </Button>
            </div>
          </Card.Content>
        </Card.Root>
      {/each}
    </div>
  {/if}
</div>

<ScanResults bind:open={sheetOpen} result={scanResult} onImported={refresh} />
