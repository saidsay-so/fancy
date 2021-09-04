<script lang="ts">
  import type { SvelteComponent } from "svelte/internal";

  import DefaultButton from "./DefaultButton.svelte";

  export let views: Array<{
    name: string;
    icon: string;
    component: SvelteComponent;
  }>;
  export let currentView;
</script>

<div class="flex flex-col px-4 py-6 border-r-2 h-full bg-gray-800">
  <ul>
    {#each views as view (view.name)}
      <li
        class="py-2 flex-1"
        on:click={() => {
          console.log(view);
          currentView = view.component;
        }}
      >
        <DefaultButton
          ><i class="w-full material-icons align-middle text-3xl">{view.icon}</i
          >
          {view.name}
        </DefaultButton>
      </li>
    {/each}
  </ul>
</div>
