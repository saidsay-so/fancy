<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import type { SvelteComponent, SvelteComponentDev } from 'svelte/internal'

  const pageDispatcher = createEventDispatcher()

  export let childName: string | null = null
  export let parentName: string = ''
  export let component: SvelteComponent | SvelteComponentDev

  const switchToParent = () => pageDispatcher('page', component)
</script>

<header class="breadcrumbs">
  <ul>
    <li class="text-center">
      <h2 class="link" on:click={switchToParent}>{parentName}</h2>
    </li>
    {#if childName}
      <li class="text-center"><h2>{childName}</h2></li>
    {/if}
  </ul>
</header>
