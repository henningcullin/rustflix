<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Sheet from '$lib/components/ui/sheet';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { Trash2 } from '$lib/lucide';
  import AdminCell from './AdminCell.svelte';
  import { columnLabel, type TableConfig } from './tables';

  type Props = {
    open: boolean;
    config: TableConfig;
    row: Record<string, unknown> | null;
    onClose: () => void;
    onSave: (patch: Record<string, unknown>) => Promise<void>;
    onDelete: () => Promise<void>;
  };

  let {
    open = $bindable(),
    config,
    row,
    onClose,
    onSave,
    onDelete,
  }: Props = $props();

  let confirmDeleteOpen = $state(false);
  let deleting = $state(false);

  async function handleCellSave(columnKey: string, next: unknown) {
    if (!row) {
      return;
    }
    await onSave({ [columnKey]: next });
    row[columnKey] = next;
  }

  async function handleDelete() {
    if (!row) {
      return;
    }
    deleting = true;
    try {
      await onDelete();
      confirmDeleteOpen = false;
      onClose();
    } finally {
      deleting = false;
    }
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content side="right" class="w-full overflow-y-auto sm:max-w-lg">
    <Sheet.Header>
      <Sheet.Title>{config.label} · row</Sheet.Title>
      <Sheet.Description>
        {row
          ? config.primaryKey.map((pk) => `${pk}=${row[pk]}`).join(', ')
          : ''}
      </Sheet.Description>
    </Sheet.Header>

    {#if row}
      <div class="mt-6 flex flex-col gap-4">
        {#each config.columns as column (column.key)}
          <div class="flex flex-col gap-1">
            <span class="text-xs uppercase tracking-wide text-muted-foreground">
              {columnLabel(column)}
            </span>
            <div class="min-h-[2rem]">
              <AdminCell
                {column}
                value={row[column.key]}
                onSave={(next) => handleCellSave(column.key, next)}
              />
            </div>
          </div>
        {/each}
      </div>

      <Sheet.Footer class="mt-8 flex items-center justify-between gap-2">
        <AlertDialog.Root bind:open={confirmDeleteOpen}>
          <AlertDialog.Trigger
            class="inline-flex h-9 items-center justify-center gap-2 rounded-md bg-destructive px-3 py-2 text-sm font-medium text-destructive-foreground shadow-sm transition-colors hover:bg-destructive/90"
          >
            <Trash2 class="size-4" />
            Delete row
          </AlertDialog.Trigger>
          <AlertDialog.Content>
            <AlertDialog.Header>
              <AlertDialog.Title>Delete this row?</AlertDialog.Title>
              <AlertDialog.Description>
                {config.label} ·
                {config.primaryKey.map((pk) => `${pk}=${row?.[pk]}`).join(', ')}
              </AlertDialog.Description>
            </AlertDialog.Header>
            <AlertDialog.Footer>
              <AlertDialog.Cancel disabled={deleting}>Cancel</AlertDialog.Cancel>
              <AlertDialog.Action
                variant="destructive"
                onclick={handleDelete}
                disabled={deleting}
              >
                {deleting ? 'Deleting…' : 'Delete'}
              </AlertDialog.Action>
            </AlertDialog.Footer>
          </AlertDialog.Content>
        </AlertDialog.Root>

        <Button variant="ghost" onclick={onClose}>Close</Button>
      </Sheet.Footer>
    {/if}
  </Sheet.Content>
</Sheet.Root>
