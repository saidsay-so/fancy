<script lang="ts">
  import RangeInput from '@/components/RangeInput.svelte'
  import FanConfigCard from '@/components/editor/FanConfigCard.svelte'
  import { fansNumber, fans } from '../../stores/editor/fans'
</script>

<div class="form-control">
  <label for="fans_number" class="label"
    ><span class="label-text">Fans number</span></label
  >
  <RangeInput min={1} bind:value={$fansNumber} range={1} />
</div>

<div class="grid grid-cols-3 gap-4 my-4">
  {@debug $fans}
  {#each $fans as fan, i (i)}
    <FanConfigCard
      {...fan}
      FanDisplayName={fan.FanDisplayName ?? `Fan #${i + 1}`}
    />
  {/each}
</div>
