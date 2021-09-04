<script lang="ts">
  import { scale } from "svelte/transition";

  export let title: string;
  export let value: string = null;
</script>

<article
  transition:scale
  class="bg-white shadow rounded-lg
    px-4 py-6 flex flex-col gap-4 transition-shadow hover:shadow-lg"
>
  <div class="flex justify-center">
    <h3 class="text-center flex-1">{title}</h3>
    <slot name="top" />
  </div>
  {#if value}
    <h4 class="text-center text-lg font-light">{value}</h4>
  {:else}
    <slot name="content" />
  {/if}
  <slot name="bottom" />
</article>
