<script lang="ts">
  import { Image as ImageIcon, Pencil, Play, RotateCcw } from '$lib/lucide';
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
    onTitleSave?: (next: string) => Promise<void> | void;
    onPosterChange?: () => Promise<void> | void;
    onPosterReset?: () => Promise<void> | void;
    posterIsManual?: boolean;
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
    onTitleSave,
    onPosterChange,
    onPosterReset,
    posterIsManual = false,
  }: Props = $props();

  const isTitleEditable = $derived(typeof onTitleSave === 'function');
  const isPosterEditable = $derived(typeof onPosterChange === 'function');

  let editingTitle = $state(false);
  let draftTitle = $state('');
  let saving = $state(false);

  function startEdit() {
    if (!isTitleEditable || editingTitle) {
      return;
    }
    draftTitle = title;
    editingTitle = true;
  }

  function cancelEdit() {
    editingTitle = false;
    draftTitle = '';
  }

  async function commitEdit() {
    if (!onTitleSave) {
      cancelEdit();
      return;
    }

    const next = draftTitle.trim();
    if (next.length === 0 || next === title) {
      cancelEdit();
      return;
    }

    saving = true;
    try {
      await onTitleSave(next);
    } finally {
      saving = false;
      editingTitle = false;
      draftTitle = '';
    }
  }

  function autoFocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<section class="group/hero relative isolate overflow-hidden">
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

    {#if isPosterEditable}
      <div
        class="absolute right-4 top-4 flex gap-2 opacity-0 transition-opacity duration-200 group-hover/hero:opacity-100 focus-within:opacity-100"
      >
        <button
          type="button"
          onclick={() => onPosterChange?.()}
          class="inline-flex items-center gap-1.5 rounded-md bg-background/80 px-3 py-1.5 text-xs font-medium text-foreground backdrop-blur-sm transition hover:bg-background"
        >
          <ImageIcon class="size-3.5" />
          {backdrop ? 'Change poster' : 'Add poster'}
        </button>
        {#if posterIsManual && onPosterReset}
          <button
            type="button"
            onclick={() => onPosterReset?.()}
            class="inline-flex items-center gap-1.5 rounded-md bg-background/80 px-3 py-1.5 text-xs font-medium text-muted-foreground backdrop-blur-sm transition hover:bg-background hover:text-foreground"
          >
            <RotateCcw class="size-3.5" />
            Reset to auto
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <div class="absolute inset-0 flex items-center px-6 lg:px-12">
    <div class="max-w-2xl">
      {#if subtitle}
        <div class="mb-2 text-xs font-semibold uppercase tracking-widest text-primary">
          {subtitle}
        </div>
      {/if}

      {#if editingTitle}
        <input
          use:autoFocus
          bind:value={draftTitle}
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
          class="w-full max-w-xl rounded-md border border-primary/60 bg-background/60 px-3 py-1.5 text-4xl font-bold tracking-tight text-foreground outline-none backdrop-blur sm:text-5xl lg:text-6xl"
        />
      {:else if isTitleEditable}
        <button
          type="button"
          onclick={startEdit}
          aria-label="Edit title"
          class="group/title -mx-2 flex items-center gap-3 rounded-md px-2 py-0.5 text-left text-4xl font-bold tracking-tight text-foreground transition hover:bg-background/30 sm:text-5xl lg:text-6xl"
        >
          <span>{title}</span>
          <Pencil class="size-5 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover/title:opacity-100" />
        </button>
      {:else}
        <h1 class="text-4xl font-bold tracking-tight text-foreground sm:text-5xl lg:text-6xl">
          {title}
        </h1>
      {/if}

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
