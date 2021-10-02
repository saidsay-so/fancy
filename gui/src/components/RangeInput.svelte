<script lang="ts">
  export let value = 1000
  export let timeout = 500
  export let range = 100
  export let min = 0

  const increment = () => value++
  const decrement = () => value--

  const longIncrement = () => (value += range)
  const longDecrement = () => {
    if (value - range >= min) value -= range
  }
</script>

<div class="relative w-min">
  <input
    type="number"
    bind:value
    name="poll_interval"
    class="input input-bordered pl-16 pr-16"
  />
  <button on:click={longDecrement} class="btn-secondary absolute top-0 left-0"
    ><i class="material-icons">remove</i></button
  >
  <button on:click={longIncrement} class="btn-secondary absolute top-0 right-0"
    ><i class="material-icons">add</i></button
  >
</div>

<style>
  input::-webkit-inner-spin-button {
    display: none;
  }
</style>
