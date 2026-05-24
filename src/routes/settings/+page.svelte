<script lang="ts">
  import { ChevronRight } from '$lib/lucide';
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <h1 class="mb-6 text-3xl font-bold tracking-tight">Settings</h1>
  <ul class="flex flex-col gap-2">
    <li>
      <a
        href="/settings/libraries"
        class="flex items-center justify-between rounded-md border border-border bg-card px-4 py-3 transition-colors hover:bg-accent"
      >
        <div>
          <div class="font-medium">Libraries</div>
          <div class="text-sm text-muted-foreground">
            Folders Rustflix scans for movies and series.
          </div>
        </div>
        <ChevronRight class="size-4 text-muted-foreground" />
      </a>
    </li>
    <li>
      <a
        href="/settings/metadata"
        class="flex items-center justify-between rounded-md border border-border bg-card px-4 py-3 transition-colors hover:bg-accent"
      >
        <div>
          <div class="font-medium">Metadata</div>
          <div class="text-sm text-muted-foreground">
            TMDB integration for posters, cast, and overviews.
          </div>
        </div>
        <ChevronRight class="size-4 text-muted-foreground" />
      </a>
    </li>
  </ul>
</div>
