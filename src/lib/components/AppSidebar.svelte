<script lang="ts">
  import * as Sidebar from '$lib/components/ui/sidebar/index';
  import {
    Clapperboard,
    House,
    Library,
    Settings,
  } from '$lib/lucide';
  import type { Component } from 'svelte';

  const sidebar = Sidebar.useSidebar();

  function closeSidebar() {
    sidebar.setOpen(false);
  }

  type SidebarItem = {
    title: string;
    url: string;
    icon: Component;
  };

  const items: SidebarItem[] = [
    { title: 'Home', url: '/', icon: House },
    { title: 'Films', url: '/films', icon: Clapperboard },
    { title: 'Library', url: '/library', icon: Library },
    { title: 'Settings', url: '/settings', icon: Settings },
  ];
</script>

<Sidebar.Root>
  <Sidebar.Header />
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each items as item (item.title)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton>
                {#snippet child({ props }: { props: Record<string, unknown> })}
                  <a href={item.url} {...props} onclick={closeSidebar}>
                    <item.icon />
                    <span>{item.title}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
  <Sidebar.Footer />
</Sidebar.Root>
