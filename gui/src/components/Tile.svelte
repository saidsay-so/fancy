<script lang="ts">
  import { scale } from "svelte/transition";

  export let title: string;
  export let value: string = null;
  export let danger = false;
</script>

<article
  in:scale
  class="shadow rounded-lg bg-white
    px-4 py-6 flex flex-col gap-4 transition-all hover:shadow-lg"
>
  <div class="flex justify-center">
    <h3 class="text-center text-gray-400 flex-1">{title}</h3>
    <slot name="top" />
  </div>
  {#if value}
    <h4 class="text-center text-2xl font-extrabold" class:danger>
      {value}
    </h4>
  {/if}
  <slot name="content" />
</article>

<style scoped>
  .danger {
    @apply text-red-500;
  }
</style>
