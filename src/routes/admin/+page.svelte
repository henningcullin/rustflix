<script lang="ts">
  import { ChevronRight } from '$lib/lucide';
  import { TABLES } from '$lib/admin/tables';

  const entries = Object.values(TABLES);
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <h1 class="mb-2 text-3xl font-bold tracking-tight">Admin</h1>
  <p class="mb-6 text-sm text-muted-foreground">
    Direct access to every persistent table. Use with care.
  </p>

  {#if entries.length === 0}
    <p class="text-sm text-muted-foreground">No tables configured yet.</p>
  {:else}
    <ul class="flex flex-col gap-2">
      {#each entries as config (config.id)}
        <li>
          <a
            href={`/admin/${config.id}`}
            class="flex items-center justify-between rounded-md border border-border bg-card px-4 py-3 transition-colors hover:bg-accent"
          >
            <div>
              <div class="font-medium">{config.label}</div>
              <div class="text-sm text-muted-foreground">{config.id}</div>
            </div>
            <ChevronRight class="size-4 text-muted-foreground" />
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</div>
