<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import * as Card from '$lib/components/ui/card/index';
  import { Plus, RefreshCw, Trash2 } from '$lib/lucide';
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
  let scanning = $state<number | null>(null);
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
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string') {
        await addDirectory(selected);
        await refresh();
      }
    } catch (err) {
      error = String(err);
    }
  }

  async function remove(dir: Directory) {
    try {
      await deleteDirectory(dir.id);
      directories = directories.filter((d) => d.id !== dir.id);
    } catch (err) {
      error = String(err);
    }
  }

  async function scan(dir: Directory) {
    scanning = dir.id;
    error = null;
    try {
      scanResult = await scanDirectory(dir.id);
      sheetOpen = true;
    } catch (err) {
      error = String(err);
    } finally {
      scanning = null;
    }
  }

  function onImported() {
    if (scanResult) {
      void (async () => {
        const refreshed = await scanDirectory(
          directories.find((d) => true)?.id ?? 0,
        ).catch(() => null);
        if (refreshed) scanResult = refreshed;
      })();
    }
  }
</script>

<div class="p-6">
  <div class="flex items-center justify-between mb-4">
    <h1 class="text-2xl font-bold">Directories</h1>
    <Button onclick={pickAndAdd}>
      <Plus class="size-4" /> Add directory
    </Button>
  </div>

  {#if error}
    <p class="text-red-600 text-sm mb-4">{error}</p>
  {/if}

  {#if loading}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if directories.length === 0}
    <p class="text-sm text-muted-foreground">
      No directories registered yet. Click <em>Add directory</em> to pick a folder.
    </p>
  {:else}
    <div class="space-y-3">
      {#each directories as dir (dir.id)}
        <Card.Root>
          <Card.Content class="flex items-center justify-between gap-3 p-4">
            <div class="min-w-0">
              <div class="font-medium truncate">{dir.path}</div>
              <div class="text-xs text-muted-foreground">
                {dir.recursive ? 'Recursive' : 'Top level only'}
              </div>
            </div>
            <div class="flex gap-2">
              <Button
                variant="secondary"
                size="sm"
                onclick={() => scan(dir)}
                disabled={scanning === dir.id}
              >
                <RefreshCw class={scanning === dir.id ? 'size-4 animate-spin' : 'size-4'} />
                {scanning === dir.id ? 'Scanning…' : 'Scan'}
              </Button>
              <Button variant="ghost" size="sm" onclick={() => remove(dir)}>
                <Trash2 class="size-4" />
              </Button>
            </div>
          </Card.Content>
        </Card.Root>
      {/each}
    </div>
  {/if}
</div>

<ScanResults bind:open={sheetOpen} result={scanResult} {onImported} />
