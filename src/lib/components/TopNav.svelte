<script lang="ts">
  import { page } from '$app/stores';
  import { Film, House, Settings, Tv } from '$lib/lucide';

  const links = [
    { href: '/', label: 'Home', icon: House },
    { href: '/films', label: 'Movies', icon: Film },
    { href: '/series', label: 'Series', icon: Tv },
  ];

  function isActive(href: string, current: string): boolean {
    if (href === '/') return current === '/';
    return current === href || current.startsWith(href + '/');
  }
</script>

<header
  class="sticky top-0 z-40 flex h-14 items-center gap-6 border-b border-border/40 bg-background/80 px-6 backdrop-blur-md"
>
  <a href="/" class="flex items-center gap-2">
    <span class="text-xl font-extrabold tracking-tight text-primary">RUSTFLIX</span>
  </a>

  <nav class="flex items-center gap-1 text-sm">
    {#each links as link (link.href)}
      {@const active = isActive(link.href, $page.url.pathname)}
      <a
        href={link.href}
        class="rounded-md px-3 py-1.5 transition-colors {active
          ? 'text-foreground'
          : 'text-muted-foreground hover:text-foreground'}"
      >
        {link.label}
      </a>
    {/each}
  </nav>

  <div class="ml-auto flex items-center gap-2">
    <a
      href="/settings/libraries"
      class="inline-flex h-9 w-9 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
      aria-label="Library settings"
    >
      <Settings class="size-4" />
    </a>
  </div>
</header>
