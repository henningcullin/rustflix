<script lang="ts">
  import * as Sidebar from '$lib/components/ui/sidebar/index';
  import AppSidebar from '$lib/components/AppSidebar.svelte';

  import '../app.css';

  let { children } = $props();
</script>

<Sidebar.Provider open={false}>
  <AppSidebar />
  <main>
    <Sidebar.Trigger />
    {@render children()}
  </main>
</Sidebar.Provider>
