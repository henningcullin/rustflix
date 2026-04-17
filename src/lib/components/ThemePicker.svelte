<script lang="ts">
  import { Monitor, Moon, Sun } from '$lib/lucide';
  import type { Theme } from '$lib/types';
  import type { Component } from 'svelte';

  let { value = $bindable<Theme>('system'), onchange }: {
    value?: Theme;
    onchange?: (theme: Theme) => void;
  } = $props();

  type Option = { id: Theme; label: string; icon: Component; hint: string };

  const options: Option[] = [
    { id: 'system', label: 'System', icon: Monitor, hint: 'Match my OS' },
    { id: 'light', label: 'Light', icon: Sun, hint: 'Always light' },
    { id: 'dark', label: 'Dark', icon: Moon, hint: 'Always dark' },
  ];

  function pick(t: Theme) {
    value = t;
    onchange?.(t);
  }
</script>

<div class="grid grid-cols-3 gap-3" role="radiogroup" aria-label="Theme">
  {#each options as opt (opt.id)}
    <button
      type="button"
      role="radio"
      aria-checked={value === opt.id}
      onclick={() => pick(opt.id)}
      class="flex flex-col items-center gap-2 p-4 rounded-lg border text-sm transition
             {value === opt.id ? 'border-primary ring-2 ring-primary bg-accent' : 'border-border hover:bg-accent/50'}"
    >
      <opt.icon class="size-6" />
      <span class="font-medium">{opt.label}</span>
      <span class="text-xs text-muted-foreground">{opt.hint}</span>
    </button>
  {/each}
</div>
