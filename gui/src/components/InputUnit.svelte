<script lang="ts">
  export let value = 1000
  export let unit = '%'
  export let min = 0
  export let max = 10000
  export let name = ''
</script>

<div class="relative w-min">
  <label class="input-group input-group-md">
    <input
      type="number"
      {min}
      {max}
      {name}
      {value}
      class="input input-bordered input-md"
    />
    <span>{unit}</span>
  </label>
</div>

<style>
  input::-webkit-inner-spin-button {
    display: none;
  }
</style>
