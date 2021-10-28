<script lang="ts">
  import { fade } from 'svelte/transition'

  import Sidebar from './components/Sidebar.svelte'
  import Editor from '@/views/Editor.svelte'
  import Dashboard from '@/views/Dashboard.svelte'
  import ErrorBanner from '@/components/ErrorBanner.svelte'
  import { lastError } from './stores/error'

  let currentView = Dashboard
  const views = [
    {
      name: 'Dashboard',
      icon: 'dashboard',
      component: Dashboard,
    },
    {
      name: 'Configuration Editor',
      icon: 'edit',
      component: Editor,
    },
  ]

  function handlePageChange(page: { detail: typeof Editor }) {
    currentView = page.detail
  }
</script>

<div class="flex gap-2 overflow-x-hidden">
  <aside class="h-screen fixed w-52">
    <Sidebar {views} bind:currentView />
  </aside>
  <main class="ml-52 h-screen flex-1">
    <div class="px-4 py-2">
      <svelte:component this={currentView} on:page={handlePageChange} />

      <div
        transition:fade={{ duration: 1000 }}
        class="absolute ml-52 flex justify-center z-10 m-auto left-0 right-0 bottom-8"
      >
        <div class="px-4">
          <ErrorBanner
            visible={$lastError}
            message={$lastError?.message}
            fatal={$lastError?.fatal ?? false}
          />
        </div>
      </div>
    </div>
  </main>
</div>
