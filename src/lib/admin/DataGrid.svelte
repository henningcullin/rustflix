<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { ChevronDown, ChevronRight, RefreshCw, Trash2 } from '$lib/lucide';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import AdminCell from './AdminCell.svelte';
  import RowDrawer from './RowDrawer.svelte';
  import { columnLabel, type ColumnConfig, type TableConfig } from './tables';

  type Props = {
    config: TableConfig;
  };

  let { config }: Props = $props();

  let rows = $state<Record<string, unknown>[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selected = $state(new Set<string>());
  let confirmBulkOpen = $state(false);
  let confirmTyped = $state('');
  let cascadeCounts = $state<{ shows: number; movies: number; episodes: number } | null>(null);
  let deleting = $state(false);

  const visibleColumns = $derived(
    config.columns.filter((column) => !column.hideInGrid),
  );

  const searchQuery = $derived(($page.url.searchParams.get('q') ?? '').trim());
  const sortColumn = $derived(
    $page.url.searchParams.get('sort') ?? config.defaultSort.column,
  );
  const sortDirection = $derived<'asc' | 'desc'>(
    ($page.url.searchParams.get('dir') as 'asc' | 'desc') ??
      config.defaultSort.direction,
  );

  const filteredRows = $derived(filterRows(rows, searchQuery));
  const sortedRows = $derived(sortRows(filteredRows, sortColumn, sortDirection));

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

  function filterRows(
    candidates: Record<string, unknown>[],
    query: string,
  ): Record<string, unknown>[] {
    if (query.length === 0) {
      return candidates;
    }
    const lowered = query.toLowerCase();
    return candidates.filter((row) =>
      JSON.stringify(row).toLowerCase().includes(lowered),
    );
  }

  function sortRows(
    candidates: Record<string, unknown>[],
    column: string,
    direction: 'asc' | 'desc',
  ): Record<string, unknown>[] {
    const sorted = [...candidates];
    sorted.sort((a, b) => {
      const av = a[column];
      const bv = b[column];
      const cmp = compareValues(av, bv);
      return direction === 'asc' ? cmp : -cmp;
    });
    return sorted;
  }

  function compareValues(a: unknown, b: unknown): number {
    if (a === null || a === undefined) {
      return b === null || b === undefined ? 0 : -1;
    }
    if (b === null || b === undefined) {
      return 1;
    }
    if (typeof a === 'number' && typeof b === 'number') {
      return a - b;
    }
    return String(a).localeCompare(String(b));
  }

  function setQueryParam(key: string, value: string | null) {
    const url = new URL($page.url);
    if (value === null || value === '') {
      url.searchParams.delete(key);
    } else {
      url.searchParams.set(key, value);
    }
    void goto(url.pathname + url.search, { replaceState: true, keepFocus: true });
  }

  function setSearch(query: string) {
    setQueryParam('q', query);
  }

  function toggleSort(column: ColumnConfig) {
    if (sortColumn === column.key) {
      setQueryParam('dir', sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      const url = new URL($page.url);
      url.searchParams.set('sort', column.key);
      url.searchParams.set('dir', 'asc');
      void goto(url.pathname + url.search, { replaceState: true, keepFocus: true });
    }
  }

  function toggleSelected(row: Record<string, unknown>) {
    const key = pkStringFor(row);
    const next = new Set(selected);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    selected = next;
  }

  function toggleSelectAll() {
    if (selected.size === sortedRows.length) {
      selected = new Set();
    } else {
      selected = new Set(sortedRows.map(pkStringFor));
    }
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

  async function openBulkDeleteConfirm() {
    confirmTyped = '';
    cascadeCounts = null;
    confirmBulkOpen = true;

    if (config.id === 'libraries' && selected.size > 0) {
      // Count cascaded rows before showing the confirm dialog.
      try {
        const allShows = await api.adminListRows('shows');
        const allMovies = await api.adminListRows('movies');
        const allEpisodes = await api.adminListRows('episodes');
        const selectedIds = new Set(selected);
        const showsCount = allShows.filter((row) =>
          selectedIds.has(String(row.library_id)),
        ).length;
        const moviesCount = allMovies.filter((row) =>
          selectedIds.has(String(row.library_id)),
        ).length;
        const showIdsToBeDeleted = new Set(
          allShows
            .filter((row) => selectedIds.has(String(row.library_id)))
            .map((row) => String(row.id)),
        );
        const episodesCount = allEpisodes.filter((row) =>
          showIdsToBeDeleted.has(String(row.show_id)),
        ).length;
        cascadeCounts = {
          shows: showsCount,
          movies: moviesCount,
          episodes: episodesCount,
        };
      } catch (caught) {
        error = String(caught);
      }
    }
  }

  async function bulkDelete() {
    if (config.id === 'libraries' && cascadeCounts) {
      // Typed-confirmation: the user must type the library's path to confirm.
      const selectedRows = sortedRows.filter((row) =>
        selected.has(pkStringFor(row)),
      );
      if (selectedRows.length === 1) {
        const expected = String(selectedRows[0].path);
        if (confirmTyped !== expected) {
          return;
        }
      } else if (confirmTyped !== 'delete') {
        return;
      }
    }

    deleting = true;
    try {
      const pks = sortedRows
        .filter((row) => selected.has(pkStringFor(row)))
        .map(pkValuesFor);
      await api.adminDeleteRows(config.id, pks);
      selected = new Set();
      confirmBulkOpen = false;
      await load();
    } catch (caught) {
      error = String(caught);
    } finally {
      deleting = false;
    }
  }

  const allSelected = $derived(
    sortedRows.length > 0 && selected.size === sortedRows.length,
  );
</script>

<div class="flex flex-wrap items-center justify-between gap-3 px-6 pt-6 pb-3">
  <div class="flex items-baseline gap-3">
    <h1 class="text-2xl font-bold tracking-tight">{config.label}</h1>
    <span class="text-xs text-muted-foreground">{rows.length} rows</span>
  </div>
  <div class="flex items-center gap-2">
    <Input
      placeholder="Search…"
      value={searchQuery}
      oninput={(event) => setSearch((event.target as HTMLInputElement).value)}
      class="h-8 w-48"
    />
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
</div>

{#if selected.size > 0}
  <div class="mx-6 mb-3 flex items-center justify-between rounded-md border border-border bg-card px-3 py-2">
    <span class="text-sm">
      {selected.size} selected
    </span>
    <Button
      variant="destructive"
      size="sm"
      onclick={openBulkDeleteConfirm}
    >
      <Trash2 class="size-4" />
      Delete {selected.size}
    </Button>
  </div>
{/if}

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
  {:else if sortedRows.length === 0}
    <p class="text-sm text-muted-foreground">
      {searchQuery.length > 0 ? 'No rows match the search.' : 'No rows.'}
    </p>
  {:else}
    <table class="w-full border-collapse text-sm">
      <thead>
        <tr class="border-b border-border text-xs uppercase tracking-wide text-muted-foreground">
          <th class="w-8 px-2 py-2">
            <input
              type="checkbox"
              checked={allSelected}
              onchange={toggleSelectAll}
              aria-label="Select all"
            />
          </th>
          {#each visibleColumns as column (column.key)}
            <th class="px-2 py-2 text-left font-medium">
              <button
                type="button"
                onclick={() => toggleSort(column)}
                class="inline-flex items-center gap-1 transition-colors hover:text-foreground"
              >
                {columnLabel(column)}
                {#if sortColumn === column.key}
                  {#if sortDirection === 'asc'}
                    <ChevronDown class="size-3 rotate-180" />
                  {:else}
                    <ChevronDown class="size-3" />
                  {/if}
                {/if}
              </button>
            </th>
          {/each}
          <th class="px-2 py-2 text-right font-medium">Open</th>
        </tr>
      </thead>
      <tbody>
        {#each sortedRows as row (pkStringFor(row))}
          {@const isChecked = selected.has(pkStringFor(row))}
          <tr class="border-b border-border/50 hover:bg-accent/20">
            <td class="px-2 py-1.5">
              <input
                type="checkbox"
                checked={isChecked}
                onchange={() => toggleSelected(row)}
                aria-label="Select row"
              />
            </td>
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
                class="inline-flex items-center gap-0.5 text-xs text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
              >
                Edit
                <ChevronRight class="size-3" />
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

<AlertDialog.Root bind:open={confirmBulkOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>
        Delete {selected.size}
        {selected.size === 1 ? 'row' : 'rows'} from {config.label}?
      </AlertDialog.Title>
      <AlertDialog.Description>
        {#if config.id === 'libraries' && cascadeCounts}
          This will cascade-delete {cascadeCounts.shows} shows,
          {cascadeCounts.movies} movies, and {cascadeCounts.episodes} episodes.
          Type
          {#if selected.size === 1}
            the library's path
          {:else}
            the word "delete"
          {/if}
          to confirm.
        {:else}
          This cannot be undone.
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>

    {#if config.id === 'libraries' && cascadeCounts}
      <Input
        bind:value={confirmTyped}
        placeholder={selected.size === 1 ? 'Library path…' : 'delete'}
      />
    {/if}

    <AlertDialog.Footer>
      <AlertDialog.Cancel disabled={deleting}>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action
        variant="destructive"
        onclick={bulkDelete}
        disabled={deleting}
      >
        {deleting ? 'Deleting…' : 'Delete'}
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
