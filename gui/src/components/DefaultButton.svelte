<script lang="ts">
  export let extraClasses: string = "";
</script>

<button
  class="text-white font-light rounded outline-none transition-all bg-purple-500 px-2 py-1 hover:bg-purple-700 {extraClasses}"
>
  <slot />
</button>
