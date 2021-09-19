<script lang="ts">
  import DefaultButton from "./DefaultButton.svelte";

  export let visible = false;
</script>

<header class="bg-gradient-to-r from-red-400 to-red-600 text-white">
  {#if visible}
    <div class="flex">
      <h3
        class="flex-auto p-3 text-xl text-center align-bottom font-medium antialiased"
      >
        An error occured. You may be want to restart?
      </h3>
      <div class="p-3">
        <DefaultButton
          defaultBg="bg-white"
          hoverBg="bg-gray-400"
          extraClasses="text-black"
        >
          <i class="material-icons text-2xl">restart_alt</i>
        </DefaultButton>
      </div>
    </div>
  {/if}
</header>
