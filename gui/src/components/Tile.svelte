<script lang="ts">
  import { scale } from "svelte/transition";

  export let title: string;
  export let value: string = null;
  export let danger: boolean = false;
</script>

<article
  in:scale
  class="shadow rounded-lg
    px-4 py-6 flex flex-col gap-4 transition-all hover:shadow-lg"
  class:bg-white={!danger}
  class:danger
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

<style lang="postcss" scoped>
  .danger {
    @apply bg-gradient-to-br from-red-300 via-red-400 to-red-400 text-white;
  }
</style>
