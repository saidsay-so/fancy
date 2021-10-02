<script lang="ts">
  import { fly } from 'svelte/transition'
  import { invoke } from '@tauri-apps/api/tauri'

  export let visible = false
  export let message = ''
  export let fatal = false

  const restart = () => invoke('restart')
  const close = () => (visible = false)
  const reveal = () => (messageReveal = !messageReveal)
  let messageReveal = false
</script>

{#if visible}
  <aside class="alert text-error gap-6">
    <button class="btn-circle btn-sm btn-primary" on:click={close}>
      <i class="material-icons text-xl">close</i>
    </button>
    <div class="px-4 flex flex-col text-center">
      <div class="self-center flex gap-2">
        <div>
          <h3 class="text-xl font-medium">An error occured</h3>
          <h4>
            {#if fatal}
              You may be want to restart?
            {/if}
          </h4>
        </div>
        <button
          class="btn-xs self-center text-xl btn-circle btn-error"
          on:click={reveal}
          ><i class="material-icons"
            >{messageReveal ? 'expand_less' : 'expand_more'}</i
          ></button
        >
      </div>
      {#if message}
        {#if messageReveal}
          <p class="mt-2" transition:fly={{ y: -20, duration: 75 }}>
            {message}
          </p>
        {/if}
      {/if}
    </div>
    {#if fatal}
      <button class="btn-primary" on:click={restart}>
        Restart
        <i class="material-icons text-2xl ml-2">restart_alt</i>
      </button>
    {/if}
  </aside>
{/if}
