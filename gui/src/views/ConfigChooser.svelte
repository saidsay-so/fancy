<script lang="ts">
  import {
    getConfigsList,
    activeDetails,
    setConfig,
    filteredConfigsList,
    model,
  } from '../stores/config'
  import type { ConfigInfo } from '../stores/config'
  import { createEventDispatcher } from 'svelte'
  import { writeText } from '@tauri-apps/api/clipboard'
  import ConfigDetails from './ConfigDetails.svelte'
  import ConfigHeader from '@/components/ConfigHeader.svelte'

  const pageDispatcher = createEventDispatcher().bind(null, 'page')
  let filterModel = true
  let search = ''
  let listStore = $getConfigsList
  let filteredList = listStore

  $: if (filterModel) {
    listStore = $filteredConfigsList
  } else {
    listStore = $getConfigsList
  }

  $: filteredList = listStore.filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase())
  )

  const setDetails = (details: ConfigInfo) => {
    $activeDetails = details
    pageDispatcher(ConfigDetails)
  }

  const handleConfigChange = (path: string) => setConfig(path)

  const setModelSearch = () => {
    search = $model.trim()
    writeText(search).catch(() => {})
  }
</script>

<ConfigHeader />

<div class="flex flex-col gap-4 mt-2 mb-6">
  <div class="flex flex-row gap-2">
    <p class="font-bold">Model: {$model}</p>
    <button class="btn-xs btn-rounded" on:click={setModelSearch}
      ><i class="material-icons">content_paste</i></button
    >
  </div>
  <div class="flex gap-4">
    <label for="filterModel">Filter only recommended</label>
    <input
      name="filterModel"
      type="checkbox"
      bind:checked={filterModel}
      class="toggle toggle-primary"
    />
  </div>

  <div class="form-control">
    <input
      type="text"
      placeholder="Search"
      class="input input-primary"
      bind:value={search}
    />
  </div>
</div>

<div class="overflow-x-auto">
  <table class="table w-full table-compact">
    <thead>
      <tr>
        <th>Name</th>
        <th>Author</th>
        <th />
      </tr>
    </thead>
    <tbody>
      {#each filteredList as config (config.name)}
        <tr
          class="hover hover:cursor-pointer overflow-clip"
          on:click={setDetails.bind(null, config)}
        >
          <td>
            <h4 class="font-bold">{config.name}</h4>
          </td>
          <td
            ><h5
              class="font-light badge"
              class:badge-warning={config.author === null ||
                config.author.length === 0}
            >
              {config.author === null || config.author.length === 0
                ? 'Unknown'
                : config.author}
            </h5>
          </td>
          <td>
            <button
              class="btn-primary"
              on:click|stopPropagation={handleConfigChange.bind(
                null,
                config.name
              )}>Apply</button
            >
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
