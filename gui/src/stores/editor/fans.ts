import { derived, get, writable } from 'svelte/store'
import type { FanConfiguration, TemperatureThreshold } from 'nbfc-types'

interface EditFanConfiguration extends FanConfiguration {
  index: number
  prettyIndexName: string
}

const defaultTemperatureThresholds: TemperatureThreshold[] = [
  {
    UpThreshold: 0,
    DownThreshold: 0,
    FanSpeed: 0.0,
  },
  {
    UpThreshold: 50,
    DownThreshold: 40,
    FanSpeed: 100.0,
  },
]

const defaultFanConfiguration: (index: number) => EditFanConfiguration = (
  index: number
) => ({
  FanDisplayName: null,
  IndependentReadMinMaxValues: false,
  FanSpeedResetValue: 0,
  MaxSpeedValue: 0,
  MinSpeedValue: 0,
  MaxSpeedValueRead: 0,
  MinSpeedValueRead: 0,
  ReadRegister: 0,
  WriteRegister: 0,
  ResetRequired: false,
  TemperatureThresholds: [
    ...defaultTemperatureThresholds.map((t) => ({ ...t })),
  ],
  index,
  prettyIndexName: `Fan #${index + 1}`,
})

export const fans = {
  ...writable([{ ...defaultFanConfiguration(0) }]),
  edit: (fan: EditFanConfiguration): void =>
    fans.update((fans) => [
      ...fans.slice(0, fan.index),
      fan,
      ...fans.slice(fan.index + 1),
    ]),
  setLength: (length: number): void =>
    fans.update((fans) => {
      if (length < 1) return fans

      const oldLength = fans.length
      fans = [...fans.slice(0, length)]

      if (length > oldLength) {
        for (let i = 0; i < length - oldLength; i++) {
          const index = oldLength + i
          fans.push({
            ...defaultFanConfiguration(index),
          })
        }
      }

      return fans
    }),
}

export const editedFanConfig = {
  ...writable<EditFanConfiguration>(),
  selectFanIndex: (index: number): void => {
    const fan = get(fans)[index]
    if (fan)
      editedFanConfig.set({
        ...fan,
      })
  },
}

export const fansNumber = {
  ...derived(fans, (fans) => fans.length),
  set: (length: number): void => fans.setLength(length),
}
