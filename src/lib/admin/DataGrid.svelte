<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { RefreshCw } from '$lib/lucide';
  import AdminCell from './AdminCell.svelte';
  import RowDrawer from './RowDrawer.svelte';
  import { columnLabel, type TableConfig } from './tables';

  type Props = {
    config: TableConfig;
  };

  let { config }: Props = $props();

  let rows = $state<Record<string, unknown>[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const visibleColumns = $derived(
    config.columns.filter((column) => !column.hideInGrid),
  );

  const rowParam = $derived($page.url.searchParams.get('row'));
  const activeRow = $derived(rowParam ? findRowByPk(rows, rowParam) : null);
  let drawerOpen = $state(false);

  $effect(() => {
    if (rowParam && activeRow) {
      drawerOpen = true;
    } else if (!rowParam) {
      drawerOpen = false;
    }
  });

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

  function pkStringFor(row: Record<string, unknown>): string {
    return pkValuesFor(row).map((v) => String(v)).join(':');
  }

  function findRowByPk(
    candidates: Record<string, unknown>[],
    pkString: string,
  ): Record<string, unknown> | null {
    for (const row of candidates) {
      if (pkStringFor(row) === pkString) {
        return row;
      }
    }
    return null;
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

  function openDrawer(row: Record<string, unknown>) {
    const url = new URL($page.url);
    url.searchParams.set('row', pkStringFor(row));
    void goto(url.pathname + url.search, { replaceState: false, keepFocus: true });
  }

  function closeDrawer() {
    const url = new URL($page.url);
    url.searchParams.delete('row');
    void goto(url.pathname + (url.search ? url.search : ''), {
      replaceState: false,
      keepFocus: true,
    });
  }

  async function saveDrawer(patch: Record<string, unknown>) {
    if (!activeRow) {
      return;
    }
    try {
      await api.adminUpdateRow(config.id, pkValuesFor(activeRow), patch);
    } catch (caught) {
      error = String(caught);
      throw caught;
    }
  }

  async function deleteDrawer() {
    if (!activeRow) {
      return;
    }
    try {
      await api.adminDeleteRows(config.id, [pkValuesFor(activeRow)]);
      await load();
    } catch (caught) {
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
  <div
    class="mx-6 mb-4 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-2 text-sm text-destructive-foreground"
  >
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
          <th class="px-2 py-2 text-right font-medium">Open</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (pkStringFor(row))}
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
            <td class="px-2 py-1.5 text-right">
              <button
                type="button"
                onclick={() => openDrawer(row)}
                class="text-xs text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
              >
                Edit…
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<RowDrawer
  bind:open={drawerOpen}
  {config}
  row={activeRow}
  onClose={closeDrawer}
  onSave={saveDrawer}
  onDelete={deleteDrawer}
/>
