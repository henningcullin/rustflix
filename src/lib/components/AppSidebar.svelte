<script lang="ts">
  import * as Sidebar from '$lib/components/ui/sidebar/index';
  import { Clapperboard, House, Tv, User } from '$lib/lucide';

  import type { Component } from 'svelte';

  type SidebarItem = {
    title: string;
    url: string;
    icon: Component;
  };

  const items: SidebarItem[] = [
    {
      title: 'Home',
      url: '/',
      icon: House,
    },
    {
      title: 'Persons',
      url: '/persons',
      icon: User,
    },
    {
      title: 'Films',
      url: '/films',
      icon: Clapperboard,
    },
    {
      title: 'Series',
      url: '/series',
      icon: Tv,
    },
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
                {#snippet child({ props })}
                  <a href={item.url} {...props}>
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
