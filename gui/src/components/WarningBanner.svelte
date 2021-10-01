<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";

  export let visible = false;

  const restart = () => invoke("restart");
  const close = () => (visible = false);
</script>

{#if visible}
  <header class="alert alert-error">
    <button class="btn-sm self-stretch" on:click={close}>
      <i class="material-icons text-xl">close</i>
    </button>
    <h3 class="text-xl text-center font-medium">
      An error occured. You may be want to restart?
    </h3>
    <button on:click={restart}>
      <i class="material-icons text-2xl">restart_alt</i>
    </button>
  </header>
{/if}
