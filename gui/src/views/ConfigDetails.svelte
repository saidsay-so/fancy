<script lang="ts">
  import { activeDetails } from "../stores/config";
  import type { Threshold } from "../stores/config";
  import { createEventDispatcher } from "svelte";
  import ConfigChooser from "./ConfigChooser.svelte";

  let currentThreshold = null;
  const pageDispatcher = createEventDispatcher().bind(null, "page");

  function handleThresholdClick(threshold: Threshold) {
    currentThreshold = threshold;
  }

  const switchToConfigChooser = () => pageDispatcher(ConfigChooser);
</script>

<header class="breadcrumbs">
  <ul>
    <li class="text-center">
      <h2 class="link link-hover" on:click={switchToConfigChooser}>
        Configurations
      </h2>
    </li>
    <li class="text-center"><h2>{$activeDetails.name}</h2></li>
  </ul>
</header>

<div>
  <div class="divider">Thresholds</div>
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
        <div class="flex">
          <h3 class="flex-auto">Down threshold</h3>
          <p>
            {currentThreshold.DownThreshold} °C
          </p>
        </div>

        <div class="flex">
          <h3 class="flex-auto">Up threshold</h3>
          <p>
            {currentThreshold.UpThreshold} °C
          </p>
        </div>

        <div class="flex">
          <h3 class="flex-auto">Speed</h3>
          <p>
            {currentThreshold.FanSpeed.toFixed()} %
          </p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style lang="postcss" scoped>
  .step:hover:after {
    @apply bg-info text-base-100;
  }
</style>
