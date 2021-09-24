<script lang="ts">
  export let extraClasses = '';
</script>

<button class="btn bg-purple-500 hover:bg-purple-700 {extraClasses}" on:click>
  <slot />
</button>
