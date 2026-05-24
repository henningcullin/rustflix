<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import { cn } from '$lib/utils.js';

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: SelectPrimitive.TriggerProps = $props();
</script>

<SelectPrimitive.Trigger
  bind:ref
  class={cn(
    'border-input bg-background ring-offset-background placeholder:text-muted-foreground focus:ring-ring data-[placeholder]:text-muted-foreground flex h-9 w-full items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm shadow-sm transition-colors focus:outline-none focus:ring-1 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1',
    className
  )}
  {...restProps}
>
  {@render children?.()}
  <ChevronDown class="size-4 shrink-0 opacity-50" />
</SelectPrimitive.Trigger>
