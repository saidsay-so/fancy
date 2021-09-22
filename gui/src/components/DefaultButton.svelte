<script lang="ts">
  export let extraClasses = "";
  export let bg = "bg-purple-500";
  export let hoverBg = "bg-purple-700";
  $: hoverBg = `hover:${hoverBg}`;
</script>

<button
  class="text-white font-light rounded outline-none transition-all  px-2 py-1 {bg} {hoverBg} {extraClasses}"
  on:click
>
  <slot />
</button>
