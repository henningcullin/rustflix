<script lang="ts">
  import { Play, CheckCircle, Image as ImageIcon } from '$lib/lucide';

  type Props = {
    href: string;
    title: string;
    subtitle?: string;
    posterPath?: string | null;
    watched?: boolean;
    progressPct?: number;
    badge?: string;
  };

  let {
    href,
    title,
    subtitle,
    posterPath,
    watched = false,
    progressPct = 0,
    badge,
  }: Props = $props();
</script>

<a
  {href}
  class="group flex w-full flex-col gap-2 outline-none"
  aria-label={title}
>
  <div
    class="poster-shadow relative aspect-[2/3] w-full overflow-hidden rounded-lg bg-card ring-0 ring-primary/0 transition-all duration-200 group-hover:scale-[1.03] group-hover:ring-2 group-hover:ring-primary/60 group-focus-visible:ring-2 group-focus-visible:ring-primary"
  >
    {#if posterPath}
      <img
        src={posterPath}
        alt={title}
        class="absolute inset-0 h-full w-full object-cover"
        loading="lazy"
      />
    {:else}
      <div
        class="absolute inset-0 flex flex-col items-center justify-center bg-gradient-to-br from-zinc-800 via-zinc-900 to-black p-3 text-center"
      >
        <ImageIcon class="size-8 text-muted-foreground/40" />
        <span class="mt-2 line-clamp-3 text-xs font-medium text-muted-foreground">
          {title}
        </span>
      </div>
    {/if}

    {#if watched}
      <div class="absolute right-2 top-2 rounded-full bg-black/70 p-1 backdrop-blur-sm">
        <CheckCircle class="size-4 text-emerald-400" />
      </div>
    {/if}

    {#if badge}
      <div
        class="absolute left-2 top-2 rounded bg-primary/90 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary-foreground"
      >
        {badge}
      </div>
    {/if}

    <div
      class="absolute inset-0 flex items-end justify-center bg-gradient-to-t from-black/80 via-black/0 to-black/0 opacity-0 transition-opacity duration-200 group-hover:opacity-100"
    >
      <div
        class="mb-3 flex size-12 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg shadow-primary/30"
      >
        <Play class="size-5 fill-current" />
      </div>
    </div>

    {#if progressPct > 0 && !watched}
      <div class="absolute inset-x-0 bottom-0 h-1 bg-black/60">
        <div class="h-full bg-primary" style="width: {progressPct}%"></div>
      </div>
    {/if}
  </div>

  <div class="px-0.5">
    <div class="line-clamp-1 text-sm font-medium text-foreground">{title}</div>
    {#if subtitle}
      <div class="line-clamp-1 text-xs text-muted-foreground">{subtitle}</div>
    {/if}
  </div>
</a>
