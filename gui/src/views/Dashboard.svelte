<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Tile from "@/components/Tile.svelte";
  import Switch from "@/components/Switch.svelte";
  import {
    config,
    fansSpeeds,
    temperatures,
    meanTemperature,
    critical,
    setTargetSpeed,
    fansNames,
    targetSpeeds,
    auto,
  } from "../stores/props";
  import TilesGroup from "@/components/TilesGroup.svelte";
  import ConfigChooser from "./ConfigChooser.svelte";

  export let meanTemp = true;

  const handleSetSpeed = (
    index: number,
    ev: Event & { target: EventTarget & HTMLInputElement }
  ) => setTargetSpeed(index, Number(ev.target.value));

  const pageDispatcher = createEventDispatcher();

  const handleConfig = (page) => pageDispatcher("page", page);
</script>

<div class="flex flex-col">
  <TilesGroup>
    <Tile title="Configuration">
      <button on:click={handleConfig.bind(null, ConfigChooser)}>
        <i class="material-icons align-top text-2xl">settings</i>
      </button>
      <h4
        slot="content"
        class="text-center text-2xl font-extrabold overflow-clip"
      >
        {$config}
      </h4>
    </Tile>
  </TilesGroup>

  <div class="divider">Fans speeds</div>
  <div class="flex justify-center gap-2">
    <h4 class="text-center">Automatic speeds</h4>
    <div class="w-10 h-6">
      <Switch bind:checked={$auto} />
    </div>
  </div>

  <TilesGroup>
    {#each $fansSpeeds.map( (s, i) => [$fansNames[i], s, $targetSpeeds[i], i] ) as [name, speed, target, i] (name)}
      <Tile title={name}>
        <div slot="content">
          <h4
            class="text-center text-3xl font-extrabold"
            class:text-error={$critical}
          >
            {speed.toFixed()} %
          </h4>
          {#if !$auto}
            <input
              type="range"
              min="0"
              max="100"
              class="range"
              bind:value={target}
              on:input={handleSetSpeed.bind(null, i)}
            />
          {/if}
        </div>
      </Tile>
    {/each}
  </TilesGroup>

  <div class="divider">Temperatures</div>

  <TilesGroup>
    {#if meanTemp}
      <Tile title="Mean temperature">
        <h4
          slot="content"
          class="text-center text-4xl font-extrabold"
          class:text-warning={$critical}
        >
          {$meanTemperature.toFixed()} °C
        </h4>
      </Tile>
    {:else}
      {#each Object.entries($temperatures) as [name, value] (name)}
        <Tile title={name}>
          <h4
            slot="content"
            class="text-center text-3xl font-extrabold"
            class:text-error={$critical}
          >
            {value.toFixed()} °C
          </h4>
        </Tile>
      {/each}
    {/if}
  </TilesGroup>
</div>
