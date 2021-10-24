<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import Tile from '@/components/Tile.svelte'
  import Switch from '@/components/Switch.svelte'
  import {
    config,
    fansSpeeds,
    temperatures,
    meanTemperature,
    critical,
    fansNames,
    targetSpeeds,
    setTargetSpeed,
    auto,
  } from '../stores/props'
  import TilesGroup from '@/components/TilesGroup.svelte'
  import ConfigChooser from './ConfigChooser.svelte'

  export let meanTemp = true

  const pageDispatcher = createEventDispatcher()
  const setTargetSpeed = (index, ev) => setTargetSpeed(index, ev)
  const handleConfig = (page) => pageDispatcher('page', page)
</script>

<div class="flex flex-col gap-3">
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

  <div class="divider"><h3>Fans speeds</h3></div>

  <div class="flex justify-center gap-2">
    <h4 class="text-center">Automatic speeds</h4>
    <div class="w-10 h-6">
      <Switch bind:checked={$auto} />
    </div>
  </div>

  <div class="self-center">
    <TilesGroup>
      {#each $fansNames as name, i (name)}
        <Tile title={name}>
          <div slot="content">
            <h4
              class="text-center m-auto text-3xl font-extrabold"
              class:text-error={$critical}
            >
              {#if $fansSpeeds[i] !== undefined}
                {$fansSpeeds[i].toFixed().padStart(3, ' ')} %
              {/if}
            </h4>
            {#if !$auto}
              <input
                type="range"
                min="0"
                max="100"
                class="range"
                value={$targetSpeeds[i]}
                on:input={setTargetSpeed.bind(i)}
              />
            {/if}
          </div>
        </Tile>
      {/each}
    </TilesGroup>
  </div>

  <div class="divider"><h3>Temperatures</h3></div>

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
