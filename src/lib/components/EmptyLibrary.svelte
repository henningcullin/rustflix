<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import * as Card from '$lib/components/ui/card/index';
  import { FolderOpen } from '$lib/lucide';
  import { open } from '@tauri-apps/plugin-dialog';
  import { addDirectory, scanDirectory } from '$lib/api/directories';

  let { onAdded }: { onAdded?: () => void } = $props();

  let busy = $state(false);
  let error = $state<string | null>(null);

  async function pick() {
    busy = true;
    error = null;
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string') {
        const dir = await addDirectory(selected);
        await scanDirectory(dir.id);
        onAdded?.();
      }
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }
</script>

<Card.Root class="max-w-xl mx-auto text-center py-12">
  <Card.Content class="space-y-4">
    <div class="mx-auto size-16 rounded-full bg-accent flex items-center justify-center">
      <FolderOpen class="size-8" />
    </div>
    <div class="space-y-1">
      <h2 class="text-xl font-semibold">Your library is empty</h2>
      <p class="text-sm text-muted-foreground">
        Point Rustflix at a folder where your films live, and we'll handle the rest.
      </p>
    </div>
    <Button onclick={pick} disabled={busy}>
      {busy ? 'Working…' : 'Add a folder'}
    </Button>
    {#if error}
      <p class="text-red-600 text-sm">{error}</p>
    {/if}
  </Card.Content>
</Card.Root>
