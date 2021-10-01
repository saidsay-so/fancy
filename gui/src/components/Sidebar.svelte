<script lang="ts">
  export let views: {
    name: string;
    icon: string;
    component: unknown;
  }[];
  export let currentView: unknown;
</script>

<nav class="flex flex-col px-4 py-6 h-full bg-gray-800">
  <ul>
    {#each views as view (view.name)}
      <li class="py-2">
        <button
          on:click={() => {
            currentView = view.component;
          }}
          class="w-full"
        >
          <i class="material-icons w-full align-middle text-3xl">{view.icon}</i>
          {view.name}
        </button>
      </li>
    {/each}
  </ul>
</nav>
