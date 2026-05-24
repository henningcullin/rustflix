<script lang="ts">
  import { page } from '$app/stores';
  import DataGrid from '$lib/admin/DataGrid.svelte';
  import { TABLES, type TableId } from '$lib/admin/tables';
  import { ChevronLeft } from '$lib/lucide';

  const tableId = $derived($page.params.table as TableId);
  const config = $derived(TABLES[tableId]);
</script>

{#if config}
  <div>
    <div class="px-6 pt-4">
      <a
        href="/admin"
        class="inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        <ChevronLeft class="size-4" />
        All tables
      </a>
    </div>
    <DataGrid {config} />
  </div>
{:else}
  <div class="mx-auto max-w-3xl px-6 py-8">
    <h1 class="mb-2 text-3xl font-bold tracking-tight">Table not configured</h1>
    <p class="text-sm text-muted-foreground">
      "<code>{tableId}</code>" doesn't have an admin config yet.
    </p>
    <a class="mt-4 inline-block text-sm underline" href="/admin">Back to tables</a>
  </div>
{/if}
