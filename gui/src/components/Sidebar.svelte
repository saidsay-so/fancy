<script lang="ts">
  import DefaultButton from "./DefaultButton.svelte";

  export let views: {
    name: string;
    icon: string;
    component: unknown;
  }[];
  export let currentView: unknown;
</script>

<div class="flex flex-col px-4 py-6 border-r-2 h-full bg-gray-800">
  <ul>
    {#each views as view (view.name)}
      <li
        class="py-2"
        on:click={() => {
          currentView = view.component;
        }}
      >
        <DefaultButton extraClasses="w-full">
          <i class="material-icons w-full align-middle text-3xl">{view.icon}</i>
          {view.name}
        </DefaultButton>
      </li>
    {/each}
  </ul>
</div>
