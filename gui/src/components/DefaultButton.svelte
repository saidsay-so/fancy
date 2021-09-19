<script lang="ts">
  export let extraClasses = "";
  export let defaultBg = "bg-purple-500";
  export let hoverBg = "bg-purple-700";
</script>

<button
  class="text-white font-light rounded outline-none transition-all  px-2 py-1 {defaultBg} hover:{hoverBg} {extraClasses}"
  on:click
>
  <slot />
</button>
