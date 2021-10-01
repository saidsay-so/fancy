<script>
  import { getConfigsList, activeDetails, setConfig } from "../stores/config";
  import { createEventDispatcher } from "svelte";
  import ConfigDetails from "./ConfigDetails.svelte";

  const pageDispatcher = createEventDispatcher().bind(null, "page");

  const setDetails = (details) => {
    $activeDetails = details;
    pageDispatcher(ConfigDetails);
  };
  const handleConfig = (path) => {
    setConfig(path);
  };
</script>

<header class="breadcrumbs">
  <ul>
    <li class="text-center">
      <h2 class="link link-hover">Configurations</h2>
    </li>
  </ul>
</header>

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
      {#each $getConfigsList as config}
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
                ? "Unknown"
                : config.author}
            </h5>
          </td>
          <td>
            <button
              class="btn btn-primary pointer-events-auto"
              on:click|stopPropagation={handleConfig.bind(null, config.path)}
              >Apply</button
            >
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
