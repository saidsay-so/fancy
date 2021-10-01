<script lang="ts">
  import { scale } from "svelte/transition";

  export let title: string;
  export let value: string = null;
</script>

<article class="stat">
  <div class="stat-figure">
    <slot />
  </div>
  <div class="stat-title">
    <h3>{title}</h3>
  </div>
  <div class="stat-value">
    <div class="flex flex-col gap-4">
      {#if value}
        {value}
      {/if}
      <slot name="content" />
    </div>
  </div>
</article>
