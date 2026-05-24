<script lang="ts">
  import { Pencil } from '$lib/lucide';

  type Props = {
    title: string;
    onSave: (next: string) => Promise<void> | void;
  };

  let { title, onSave }: Props = $props();

  let editing = $state(false);
  let draft = $state('');
  let saving = $state(false);

  function startEdit() {
    if (editing) {
      return;
    }
    draft = title;
    editing = true;
  }

  function cancelEdit() {
    editing = false;
    draft = '';
  }

  async function commitEdit() {
    const next = draft.trim();
    if (next.length === 0 || next === title) {
      cancelEdit();
      return;
    }

    saving = true;
    try {
      await onSave(next);
    } finally {
      saving = false;
      editing = false;
      draft = '';
    }
  }

  function autoFocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

{#if editing}
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
    class="w-full rounded-md border border-primary/60 bg-background/60 px-2 py-1 text-sm font-medium text-foreground outline-none"
  />
{:else}
  <button
    type="button"
    onclick={startEdit}
    aria-label="Edit episode title"
    class="group/title -mx-1 flex min-w-0 items-center gap-2 rounded-md px-1 py-0.5 text-left transition hover:bg-background/30"
  >
    <span class="truncate font-medium">{title}</span>
    <Pencil
      class="size-3.5 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover/title:opacity-100"
    />
  </button>
{/if}
