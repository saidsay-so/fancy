<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';

  import DefaultButton from './DefaultButton.svelte';

  export let visible = false;
</script>

<header class="bg-red-500 text-white">
  {#if visible}
    <div class="flex">
      <h3
        class="flex-auto p-3 text-xl text-center align-bottom font-medium antialiased"
      >
        An error occured. You may be want to restart?
      </h3>
      <div class="p-3">
        <DefaultButton
          on:click={() => invoke('restart')}
          extraClasses="text-black"
        >
          <i class="material-icons text-2xl">restart_alt</i>
        </DefaultButton>
      </div>
    </div>
  {/if}
</header>
