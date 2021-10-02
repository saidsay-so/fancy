<script>
  import RangeInput from '@/components/RangeInput.svelte'
  import { fansNumber, fans } from '@/stores/editor/fans'
</script>

<div class="form-control">
  <label for="fans_number" class="label"
    ><span class="label-text">Fans number</span></label
  >
  <RangeInput min={1} bind:value={$fansNumber} range={1} />
</div>

{#each fans as fan}{/each}
