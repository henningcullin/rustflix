<script lang="ts">
  import { Play } from '$lib/lucide';
  import { formatRuntime } from '$lib/api';

  type Props = {
    title: string;
    subtitle?: string;
    overview?: string | null;
    href: string;
    playHref?: string;
    year?: number | null;
    runtime?: number | null;
    backdrop?: string | null;
  };

  let {
    title,
    subtitle,
    overview,
    href,
    playHref,
    year,
    runtime,
    backdrop,
  }: Props = $props();
</script>

<section class="relative isolate overflow-hidden">
  <div class="relative h-[60vh] min-h-[420px] w-full">
    {#if backdrop}
      <img
        src={backdrop}
        alt=""
        class="absolute inset-0 h-full w-full object-cover"
      />
    {:else}
      <div
        class="absolute inset-0 bg-[radial-gradient(ellipse_at_top_left,rgba(220,38,38,0.25),transparent_55%),radial-gradient(ellipse_at_bottom_right,rgba(59,130,246,0.18),transparent_60%)] bg-zinc-900"
      ></div>
    {/if}
    <div class="absolute inset-0 bg-gradient-to-t from-background via-background/40 to-transparent"></div>
    <div class="absolute inset-0 bg-gradient-to-r from-background/90 via-background/40 to-transparent"></div>
  </div>

  <div class="absolute inset-0 flex items-center px-6 lg:px-12">
    <div class="max-w-2xl">
      {#if subtitle}
        <div class="mb-2 text-xs font-semibold uppercase tracking-widest text-primary">
          {subtitle}
        </div>
      {/if}
      <h1 class="text-4xl font-bold tracking-tight text-foreground sm:text-5xl lg:text-6xl">
        {title}
      </h1>
      <div class="mt-3 flex items-center gap-3 text-sm text-muted-foreground">
        {#if year}<span>{year}</span>{/if}
        {#if runtime}<span>{formatRuntime(runtime)}</span>{/if}
      </div>
      {#if overview}
        <p class="mt-4 line-clamp-3 max-w-xl text-base text-muted-foreground">
          {overview}
        </p>
      {/if}
      <div class="mt-6 flex gap-3">
        <a
          href={playHref ?? href}
          class="inline-flex items-center gap-2 rounded-md bg-primary px-5 py-2.5 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/30 transition hover:bg-primary/90"
        >
          <Play class="size-4 fill-current" />
          Play
        </a>
        <a
          href={href}
          class="inline-flex items-center gap-2 rounded-md bg-secondary/70 px-5 py-2.5 text-sm font-semibold text-secondary-foreground backdrop-blur transition hover:bg-secondary"
        >
          More info
        </a>
      </div>
    </div>
  </div>
</section>
