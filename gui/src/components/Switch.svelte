<script lang="ts">
  export let active: boolean;
</script>

<div class="w-full h-full cursor-pointer" on:click={() => (active = !active)}>
  <div
    class="flex w-full h-full items-center rounded-full p-1 duration-300 ease-in-out"
    class:bg-gray-300={!active}
    class:bg-purple-500={active}
  >
    <div
      class="bg-white w-1/2 h-full rounded-full shadow-md transform duration-300 ease-in-out"
      class:translate-x-[85%]={active}
    />
  </div>
</div>
