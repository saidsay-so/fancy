<script>
  import {
    configName,
    model,
    author,
    ecPollInterval,
    readWriteWords,
    criticalTemp,
  } from '@/stores/editor/basic'
  import RangeInput from '@/components/RangeInput.svelte'
  import InputUnit from '@/components/InputUnit.svelte'

  let lockModelForm = true
</script>

<div class="form-control">
  <label for="config_name" class="label"
    ><span class="label-text">Name</span></label
  >
  <input
    type="text"
    name="config_name"
    bind:value={$configName}
    placeholder="Configuration name"
    class="input input-bordered"
  />
</div>

<div class="form-control">
  <label for="model" class="label"
    ><span class="label-text">Computer model</span></label
  >
  <div class="relative">
    <input
      disabled={lockModelForm}
      type="text"
      name="model"
      bind:value={$model}
      placeholder="Model"
      class="input input-bordered w-full pr-16"
    />
    <button
      on:click={() => (lockModelForm = !lockModelForm)}
      class="absolute top-0 right-0 btn-ghost"
      ><i class="material-icons">edit</i></button
    >
  </div>
</div>

<div class="form-control">
  <label for="author" class="label"
    ><span class="label-text">Author</span></label
  >
  <input
    type="text"
    name="author"
    bind:value={$author}
    placeholder="Author"
    class="input input-bordered"
  />
</div>

<div class="form-control">
  <label for="poll_interval" class="label"
    ><span class="label-text">Poll interval</span></label
  >
  <RangeInput bind:value={$ecPollInterval} />
</div>

<div class="form-control">
  <label for="read_words" class="label"
    ><span class="label-text">Read/write words</span></label
  >
  <input
    type="checkbox"
    checked={$readWriteWords}
    class="toggle"
    name="read_words"
  />
</div>

<div class="form-control">
  <label for="critical" class="label"
    ><span class="label-text">Critical temperature</span></label
  >
  <InputUnit bind:value={$criticalTemp} name="critical" unit="°C" min={0} />
</div>
