<script>
  import InputUnit from '@/components/InputUnit.svelte'

  export let UpThreshold
  export let DownThreshold
  export let FanSpeed
</script>

<div class="card bordered p-6 m-4">
  <div class="form-control">
    <label class="label" for="DownThreshold">
      <span class="label-text">Down threshold</span>
    </label>
    <InputUnit name="DownThreshold" unit="°C" bind:value={DownThreshold} />
  </div>

  <div class="form-control">
    <label class="label" for="UpThreshold">
      <span class="label-text">Up threshold</span>
    </label>
    <InputUnit unit="°C" bind:value={UpThreshold} />
  </div>

  <div class="form-control">
    <label class="label" for="FanSpeed">
      <span class="label-text">Fan speed</span>
    </label>
    <InputUnit unit="%" bind:value={FanSpeed} />
  </div>
</div>
