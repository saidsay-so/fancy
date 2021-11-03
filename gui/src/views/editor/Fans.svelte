<script lang="ts">
  import RangeInput from '@/components/RangeInput.svelte'
  import FanConfigCard from '@/components/editor/FanConfigCard.svelte'
  import { fansNumber, fans, editedFanConfig } from '../../stores/editor/fans'
  import FanConfig from '@/views/editor/fans/FanConfig.svelte'
  import { createEventDispatcher } from 'svelte'

  const pageDispatcher = createEventDispatcher()

  const handleEditFanConfig = (index: number) => {
    editedFanConfig.selectFanIndex(index)
    pageDispatcher('page', FanConfig)
  }
</script>

<div class="form-control">
  <label for="fans_number" class="label"
    ><span class="label-text">Fans number</span></label
  >
  <RangeInput min={1} max={255} bind:value={$fansNumber} range={1} />
</div>

<div class="grid grid-cols-3 gap-4 my-4">
  {#each $fans as fan, i (i)}
    <FanConfigCard
      {...fan}
      FanDisplayName={fan.FanDisplayName ?? `Fan #${i + 1}`}
      on:click={handleEditFanConfig.bind(null, i)}
    />
  {/each}
</div>
