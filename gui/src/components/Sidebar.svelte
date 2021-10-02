<script lang="ts">
  export let views: {
    name: string
    icon: string
    component: unknown
  }[]
  export let currentView: unknown
</script>

<nav class="px-4 py-6 h-full bg-base-200">
  <ul class="menu">
    {#each views as view (view.name)}
      <li class:bordered={currentView === view.component} class="py-2">
        <a
          on:click|preventDefault={() => {
            currentView = view.component
          }}
        >
          <i class="material-icons align-middle pr-2 text-3xl">{view.icon}</i>
          {view.name}
        </a>
      </li>
    {/each}
  </ul>
</nav>
