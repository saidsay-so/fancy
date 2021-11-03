<script>
  import Basic from '@/views/editor/Basic.svelte'
  import Fans from '@/views/editor/Fans.svelte'

  let currentComponent = Basic
  const views = [
    { component: Basic, name: 'Basic' },
    { component: Fans, name: 'Fans' },
  ]
</script>

<nav class="navbar bg-neutral text-neutral-content rounded-box">
  <button class="btn-ghost">
    <i class="material-icons pr-2">add</i>
    New
  </button>

  <button class="btn-ghost">
    <i class="material-icons pr-2">description</i>
    Open
  </button>

  <button class="btn-ghost">
    <i class="material-icons pr-2">save</i>
    Save
  </button>
</nav>

<div class="tabs my-4">
  {#each views as view (view.name)}
    <a
      class:tab-active={currentComponent === view.component}
      on:click={() => (currentComponent = view.component)}
      class="tab flex-auto tab-lg tab-bordered transition-all">{view.name}</a
    >
  {/each}
</div>

<svelte:component this={currentComponent} on:page />
