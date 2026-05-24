<script lang="ts">
  import { ChevronLeft, ChevronRight } from '$lib/lucide';
  import type { Snippet } from 'svelte';

  type Props = {
    title: string;
    href?: string;
    children: Snippet;
  };

  let { title, href, children }: Props = $props();

  let scroller: HTMLDivElement | null = $state(null);

  function scrollBy(dir: 1 | -1) {
    if (!scroller) return;
    scroller.scrollBy({ left: dir * scroller.clientWidth * 0.8, behavior: 'smooth' });
  }
</script>

<section class="mb-10">
  <div class="mb-3 flex items-end justify-between px-6">
    <h2 class="text-lg font-semibold tracking-tight">
      {#if href}
        <a href={href} class="hover:text-primary">{title}</a>
      {:else}
        {title}
      {/if}
    </h2>
    <div class="flex items-center gap-1">
      <button
        type="button"
        class="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-foreground"
        aria-label="Scroll left"
        onclick={() => scrollBy(-1)}
      >
        <ChevronLeft class="size-4" />
      </button>
      <button
        type="button"
        class="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-foreground"
        aria-label="Scroll right"
        onclick={() => scrollBy(1)}
      >
        <ChevronRight class="size-4" />
      </button>
    </div>
  </div>

  <div
    bind:this={scroller}
    class="no-scrollbar flex gap-4 overflow-x-auto overflow-y-visible scroll-smooth px-6 py-2"
  >
    {@render children()}
  </div>
</section>

<style>
  /* Each direct child becomes a fixed-width poster slot */
  div :global(> a) {
    flex: 0 0 auto;
    width: 11rem;
  }
  @media (min-width: 1024px) {
    div :global(> a) {
      width: 12rem;
    }
  }
  @media (min-width: 1400px) {
    div :global(> a) {
      width: 13rem;
    }
  }
</style>
