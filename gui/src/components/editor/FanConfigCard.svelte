<script lang="ts">
  export let FanDisplayName: string
  export let TemperatureThresholds: unknown[]
  export let ReadRegister: number
  export let WriteRegister: number

  let temperatureThresholdsCount = 0
  $: temperatureThresholdsCount = TemperatureThresholds.length
</script>

<article class="card bordered">
  <div class="card-body">
    <h4 class="card-title">{FanDisplayName}</h4>
    <div class="font-semibold">
      <p>
        Thresholds count: {temperatureThresholdsCount}
      </p>
      <p>
        Read register: {ReadRegister}
      </p>
      <p>
        Write register: {WriteRegister}
      </p>
    </div>
    <div class="card-actions justify-end">
      <button on:click class="btn-secondary">Edit</button>
    </div>
  </div>
</article>
