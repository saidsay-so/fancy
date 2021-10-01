<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import Editor from "@/views/Editor.svelte";
  import Dashboard from "@/views/Dashboard.svelte";
  import WarningBanner from "./components/WarningBanner.svelte";

  let currentView = Dashboard;
  const views = [
    {
      name: "Dashboard",
      icon: "dashboard",
      component: Dashboard,
    },
    {
      name: "Configuration Editor",
      icon: "edit",
      component: Editor,
    },
  ];

  function handlePageChange(page: { detail: typeof Editor }) {
    currentView = page.detail;
  }
</script>

<div class="flex gap-2 overflow-x-hidden">
  <aside class="h-full fixed w-44">
    <Sidebar {views} bind:currentView />
  </aside>
  <main class="ml-44 flex-1">
    <WarningBanner />
    <div class="px-4 py-2">
      <svelte:component this={currentView} on:page={handlePageChange} />
    </div>
  </main>
</div>
