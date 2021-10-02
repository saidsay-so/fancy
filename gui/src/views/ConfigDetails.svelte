<script lang="ts">
  import { activeDetails } from '../stores/config'
  import type { Threshold } from '../stores/config'
  import ConfigHeader from '@/components/ConfigHeader.svelte'

  let currentThreshold = null

  function handleThresholdClick(threshold: Threshold) {
    currentThreshold = threshold
  }
</script>

<ConfigHeader on:page name={$activeDetails.name} />

<div>
  <div class="divider"><h3>Metadata</h3></div>

  <article class="card bordered">
    <div class="card-body">
      <ul>
        <li class="flex justify-between">
          <h3>Name</h3>
          <p class="font-bold">{$activeDetails.name}</p>
        </li>

        <li class="flex justify-between">
          <h3>Model</h3>
          <p class="font-bold">{$activeDetails.model}</p>
        </li>

        <li class="flex justify-between">
          <h3>Author</h3>
          <p class="font-bold">{$activeDetails.author}</p>
        </li>
      </ul>
    </div>
  </article>

  <div class="divider"><h3>Thresholds</h3></div>

  <div class="flex p-4 flex-col gap-4">
    {#each Object.entries($activeDetails.thresholds) as [fanName, thresholds]}
      <div class="flex">
        <h3 class="font-medium">{fanName}</h3>
        <ul class="steps flex-auto">
          {#each thresholds as threshold}
            <li
              data-content={threshold.DownThreshold}
              class="step hover:cursor-pointer after:transition-colors hover:step-primary"
              class:step-primary={currentThreshold === threshold}
              on:click={handleThresholdClick.bind(null, threshold)}
            >
              {threshold.FanSpeed.toFixed()} %
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>

  {#if currentThreshold}
    <div class="card bordered mt-10">
      <div class="card-body">
        <div class="flex justify-between">
          <h3>Down threshold</h3>
          <p class="font-bold">{currentThreshold.DownThreshold} °C</p>
        </div>

        <div class="flex justify-between">
          <h3>Up threshold</h3>
          <p class="font-bold">{currentThreshold.UpThreshold} °C</p>
        </div>

        <div class="flex justify-between">
          <h3>Speed</h3>
          <p class="font-bold">{currentThreshold.FanSpeed.toFixed()} %</p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style scoped>
  .step:hover:after {
    @apply bg-info text-base-100;
  }
</style>
