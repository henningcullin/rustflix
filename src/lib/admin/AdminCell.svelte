<script lang="ts">
  import type { ColumnConfig } from './tables';
  import FkChip from './FkChip.svelte';
  import { Pencil } from '$lib/lucide';

  type Props = {
    column: ColumnConfig;
    value: unknown;
    onSave: (next: unknown) => Promise<void> | void;
  };

  let { column, value, onSave }: Props = $props();

  let editing = $state(false);
  let draft = $state('');
  let saving = $state(false);

  function display(raw: unknown): string {
    if (raw === null || raw === undefined) {
      return '';
    }
    if (column.kind === 'datetime' && typeof raw === 'number') {
      return new Date(raw * 1000).toISOString().replace('T', ' ').slice(0, 19);
    }
    if (column.kind === 'boolean') {
      return raw ? '✓' : '';
    }
    if (column.kind === 'json' && typeof raw === 'string') {
      try {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed) && parsed.length > 0 && typeof parsed[0] === 'string') {
          return parsed.join(', ');
        }
        return `${parsed.length ?? 0} items`;
      } catch {
        return raw;
      }
    }
    return String(raw);
  }

  function startEdit() {
    if (column.readonly || editing) {
      return;
    }
    if (column.kind === 'boolean') {
      void onSave(value ? 0 : 1);
      return;
    }
    draft = value === null || value === undefined ? '' : String(value);
    editing = true;
  }

  function cancelEdit() {
    editing = false;
    draft = '';
  }

  async function commitEdit() {
    const next = draft.trim();
    if (next === String(value ?? '')) {
      cancelEdit();
      return;
    }

    const payload = parseDraft(next);
    saving = true;
    try {
      await onSave(payload);
    } finally {
      saving = false;
      editing = false;
      draft = '';
    }
  }

  function parseDraft(input: string): unknown {
    if (input.length === 0) {
      return null;
    }
    if (column.kind === 'json') {
      return input;
    }
    // Heuristic: if the field name implies a number and the input parses
    // cleanly, send a number. Otherwise send a string.
    const asNumber = Number(input);
    if (
      !Number.isNaN(asNumber) &&
      Number.isFinite(asNumber) &&
      /^-?\d+(\.\d+)?$/.test(input)
    ) {
      return asNumber;
    }
    return input;
  }

  function autoFocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

{#if column.fkTable && column.fkLabel && !editing}
  <FkChip table={column.fkTable} labelColumn={column.fkLabel} {value} />
{:else if editing}
  <input
    use:autoFocus
    bind:value={draft}
    disabled={saving}
    onblur={commitEdit}
    onkeydown={(event) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        void commitEdit();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        cancelEdit();
      }
    }}
    class="w-full rounded border border-primary/60 bg-background/60 px-2 py-1 text-sm text-foreground outline-none"
  />
{:else if column.readonly}
  <span class="text-sm text-muted-foreground">{display(value)}</span>
{:else}
  <button
    type="button"
    onclick={startEdit}
    class="group/cell -mx-1 flex w-full min-w-0 items-center gap-2 rounded px-1 py-0.5 text-left text-sm transition hover:bg-background/30"
  >
    <span class="truncate">{display(value) || '—'}</span>
    <Pencil
      class="size-3 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover/cell:opacity-100"
    />
  </button>
{/if}
