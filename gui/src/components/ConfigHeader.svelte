<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import ConfigChooser from '@/views/ConfigChooser.svelte'

  const pageDispatcher = createEventDispatcher()

  export let name: string

  const switchToConfigChooser = () => pageDispatcher('page', ConfigChooser)
</script>

<header class="breadcrumbs">
  <ul>
    <li class="text-center">
      <h2 class="link" on:click={switchToConfigChooser}>Configurations</h2>
    </li>
    {#if name}
      <li class="text-center"><h2>{name}</h2></li>
    {/if}
  </ul>
</header>
