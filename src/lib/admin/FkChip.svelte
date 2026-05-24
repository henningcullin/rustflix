<script lang="ts">
  import { api } from '$lib/api';
  import type { TableId } from './tables';

  type Props = {
    table: TableId;
    labelColumn: string;
    value: unknown;
  };

  let { table, labelColumn, value }: Props = $props();

  let label = $state<string | null>(null);
  let loading = $state(false);

  $effect(() => {
    if (value === null || value === undefined) {
      label = null;
      return;
    }

    const cacheKey = `${table}:${labelColumn}:${JSON.stringify(value)}`;
    const cached = labelCache.get(cacheKey);
    if (cached !== undefined) {
      label = cached;
      return;
    }

    loading = true;
    void api
      .adminFkLabel(table, labelColumn, value)
      .then((result) => {
        labelCache.set(cacheKey, result);
        label = result;
      })
      .catch(() => {
        label = null;
      })
      .finally(() => {
        loading = false;
      });
  });
</script>

{#if value === null || value === undefined}
  <span class="text-muted-foreground">—</span>
{:else}
  <a
    href={`/admin/${table}?row=${encodeURIComponent(String(value))}`}
    class="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-2 py-0.5 text-xs text-foreground transition-colors hover:bg-accent"
  >
    <span class="font-mono text-muted-foreground">#{String(value)}</span>
    {#if loading}
      <span class="text-muted-foreground">…</span>
    {:else if label}
      <span class="max-w-[18ch] truncate">{label}</span>
    {/if}
  </a>
{/if}

<script lang="ts" module>
  const labelCache = new Map<string, string | null>();
</script>
