<script lang="ts">
  import { fly } from "svelte/transition";
  import Tile from "@/components/Tile.svelte";
  import DefaultButton from "@/components/DefaultButton.svelte";
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
  } from "../props";

  export let meanTemp = true;

  function handleSetSpeed(
    index: number,
    ev: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    return setTargetSpeed(index, Number(ev.currentTarget.value));
  }
</script>

<div class="flex flex-col gap-6" transition:fly={{ x: 250 }}>
  <Tile title="Configuration" value={$config}>
    <DefaultButton slot="top">
      <i class="material-icons align-top text-2xl">settings</i>
    </DefaultButton>
  </Tile>

  <div class="flex flex-col gap-4">
    <h2 class="text-center">Fans speeds</h2>
    <div class="flex justify-center gap-2">
      <h4 class="text-center">Automatic speeds</h4>
      <div class="w-10 h-6">
        <Switch bind:active={$auto} />
      </div>
    </div>
    <div class="flex justify-center">
      {#each $fansSpeeds.map( (s, i) => [$fansNames[i], s, $targetSpeeds[i], i] ) as [name, speed, target, i] (name)}
        <Tile title={name} value={`${speed.toFixed()} %`}>
          <div slot="content" class="flex flex-col justify-center">
            {#if !$auto}
              <input
                type="range"
                min="0"
                max="100"
                bind:value={target}
                on:input={handleSetSpeed.bind(null, i)}
              />
            {/if}
          </div>
        </Tile>
      {/each}
    </div>
  </div>

  {#if meanTemp}
    <Tile
      title="Temperature"
      danger={$critical}
      value={`${$meanTemperature.toFixed()} °C`}
    />
  {:else}
    <div class="grid grid-cols-3 gap-4">
      {#each Object.entries($temperatures) as [name, value] (name)}
        <Tile title={name} value={`${value.toFixed()} °C`} />
      {/each}
    </div>
  {/if}
</div>
