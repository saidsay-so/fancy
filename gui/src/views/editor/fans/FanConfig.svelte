<script lang="ts">
  import RangeInput from '@/components/RangeInput.svelte'
  import ChildHeader from '@/components/ChildHeader.svelte'
  import Editor from '@/views/Editor.svelte'
  import Threshold from './Threshold.svelte'
  import { editedFanConfig, fans } from '../../../stores/editor/fans'
  import { get } from 'svelte/store'
  import { createEventDispatcher } from 'svelte'

  const pageDispatcher = createEventDispatcher().bind(null, 'page', Editor)

  const handleApply = () => {
    fans.edit(get(editedFanConfig))
    pageDispatcher()
  }
</script>

<ChildHeader
  parentName="Fans"
  on:page
  childName={$editedFanConfig.FanDisplayName ||
    $editedFanConfig.prettyIndexName}
  component={Editor}
/>

<h3 class="divider">Basic configuration</h3>

<section class="flex flex-col">
  <div class="form-control">
    <label for="FanDisplayName" class="label"
      ><span class="label-text">Name</span></label
    >
    <input
      type="text"
      name="FanDisplayName"
      placeholder={$editedFanConfig.prettyIndexName}
      class="input input-bordered"
      bind:value={$editedFanConfig.FanDisplayName}
    />
  </div>

  <div class="form-control">
    <label for="Reset value" class="label"
      ><span class="label-text">Reset value</span></label
    >

    <RangeInput
      name="Reset value"
      min={0}
      max={255}
      range={1}
      bind:value={$editedFanConfig.FanSpeedResetValue}
    />
  </div>

  <div class="form-control my-2 w-[fit-content]">
    <label for="resetRequired" class="label"
      ><span class="label-text mr-2">Reset required</span>
      <input
        type="checkbox"
        name="resetRequired"
        class="toggle"
        bind:checked={$editedFanConfig.ResetRequired}
      />
    </label>
  </div>

  <div class="form-control my-2 w-[fit-content]">
    <label for="linkedReadValues" class="label"
      ><span class="label-text mr-2">Independent read min/max values</span>
      <input
        type="checkbox"
        name="linkedReadValues"
        class="toggle"
        bind:checked={$editedFanConfig.IndependentReadMinMaxValues}
      />
    </label>
  </div>

  <div class="flex gap-16">
    <div class="form-control">
      <label for="Max speed" class="label"
        ><span class="label-text">Max speed value</span></label
      >

      <RangeInput
        min={0}
        max={2 ** 16 - 1}
        range={10}
        bind:value={$editedFanConfig.MaxSpeedValue}
      />
    </div>

    {#if !$editedFanConfig.IndependentReadMinMaxValues}
      <div class="form-control">
        <label for="Max speed (read override)" class="label"
          ><span class="label-text">Max speed value (read override)</span
          ></label
        >

        <RangeInput
          min={0}
          max={2 ** 16 - 1}
          range={10}
          bind:value={$editedFanConfig.MaxSpeedValueRead}
        />
      </div>
    {/if}
  </div>

  <div class="flex gap-16">
    <div class="form-control">
      <label for="Min speed" class="label"
        ><span class="label-text">Min speed value</span></label
      >

      <RangeInput
        min={0}
        max={2 ** 16 - 1}
        range={10}
        bind:value={$editedFanConfig.MinSpeedValue}
      />
    </div>

    {#if !$editedFanConfig.IndependentReadMinMaxValues}
      <div class="form-control">
        <label for="Min speed (read override)" class="label"
          ><span class="label-text">Min speed value (read override)</span
          ></label
        >

        <RangeInput
          min={0}
          max={2 ** 16 - 1}
          range={10}
          bind:value={$editedFanConfig.MinSpeedValueRead}
        />
      </div>
    {/if}
  </div>

  <div class="form-control">
    <label for="Read register" class="label"
      ><span class="label-text">Read register</span></label
    >

    <RangeInput
      min={0}
      max={255}
      range={1}
      bind:value={$editedFanConfig.ReadRegister}
    />
  </div>

  <div class="form-control flex-1">
    <label for="Write register" class="label"
      ><span class="label-text">Write register</span></label
    >

    <RangeInput
      min={0}
      max={255}
      range={1}
      bind:value={$editedFanConfig.WriteRegister}
    />
  </div>
</section>

<h3 class="divider">Temperature thresholds</h3>

{#each $editedFanConfig.TemperatureThresholds as threshold}
  <Threshold {...threshold} />
{/each}

<div class="pt-4 flex-1 flex justify-between">
  <button class="btn-primary" on:click={handleApply}>Apply</button>
  <button on:click={pageDispatcher}>Cancel</button>
</div>
