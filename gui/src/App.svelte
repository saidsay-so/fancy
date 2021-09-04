<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import Editor from "@/views/Editor.svelte";
  import Dashboard from "@/views/Dashboard.svelte";

  let currentView = Dashboard;
  let views = [
    { name: "Dashboard", icon: "dashboard", component: Dashboard },
    { name: "Configuration Editor", icon: "edit", component: Editor },
  ];

  function handlePageChange(page) {
    currentView = page;
  }
</script>

<div class="flex gap-2">
  <aside class="h-full fixed w-44">
    <Sidebar {views} bind:currentView />
  </aside>
  <main class="ml-48 flex-1 px-4 py-2">
    <svelte:component this={currentView} on:page={handlePageChange} />
  </main>
</div>
