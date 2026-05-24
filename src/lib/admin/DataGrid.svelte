<script lang="ts">
  import { api } from '$lib/api';
  import { RefreshCw } from '$lib/lucide';
  import AdminCell from './AdminCell.svelte';
  import { columnLabel, type TableConfig } from './tables';

  type Props = {
    config: TableConfig;
  };

  let { config }: Props = $props();

  let rows = $state<Record<string, unknown>[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const visibleColumns = $derived(config.columns.filter((column) => !column.hideInGrid));

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    error = null;
    try {
      rows = await api.adminListRows(
        config.id,
        config.defaultSort.column,
        config.defaultSort.direction,
      );
    } catch (caught) {
      error = String(caught);
    } finally {
      loading = false;
    }
  }

  function pkValuesFor(row: Record<string, unknown>): unknown[] {
    return config.primaryKey.map((column) => row[column]);
  }

  async function saveCell(
    row: Record<string, unknown>,
    columnKey: string,
    next: unknown,
  ) {
    const previous = row[columnKey];
    row[columnKey] = next;
    try {
      await api.adminUpdateRow(config.id, pkValuesFor(row), {
        [columnKey]: next,
      });
    } catch (caught) {
      row[columnKey] = previous;
      error = String(caught);
    }
  }
</script>

<div class="flex items-center justify-between gap-3 px-6 pt-6 pb-3">
  <h1 class="text-2xl font-bold tracking-tight">{config.label}</h1>
  <button
    type="button"
    onclick={load}
    disabled={loading}
    class="inline-flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground disabled:opacity-50"
    aria-label="Refresh"
  >
    <RefreshCw class="size-4 {loading ? 'animate-spin' : ''}" />
  </button>
</div>

{#if error}
  <div class="mx-6 mb-4 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-2 text-sm text-destructive-foreground">
    {error}
  </div>
{/if}

<div class="overflow-x-auto px-6 pb-8">
  {#if loading}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if rows.length === 0}
    <p class="text-sm text-muted-foreground">No rows.</p>
  {:else}
    <table class="w-full border-collapse text-sm">
      <thead>
        <tr class="border-b border-border text-xs uppercase tracking-wide text-muted-foreground">
          {#each visibleColumns as column (column.key)}
            <th class="px-2 py-2 text-left font-medium">{columnLabel(column)}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as row (config.primaryKey.map((k) => row[k]).join(':'))}
          <tr class="border-b border-border/50 hover:bg-accent/20">
            {#each visibleColumns as column (column.key)}
              <td class="px-2 py-1.5 align-middle">
                <AdminCell
                  {column}
                  value={row[column.key]}
                  onSave={(next) => saveCell(row, column.key, next)}
                />
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
