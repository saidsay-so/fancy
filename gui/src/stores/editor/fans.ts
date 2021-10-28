import { derived, writable } from 'svelte/store'
import type { FanConfiguration } from 'nbfc-types'

const defaultFanConfiguration: FanConfiguration = {
  FanDisplayName: null,
  IndependentReadMinMaxValues: false,
  MaxSpeedValue: 0,
  MinSpeedValue: 0,
  MaxSpeedValueRead: 0,
  MinSpeedValueRead: 0,
  ReadRegister: 0,
  WriteRegister: 0,
  ResetRequired: false,
  TemperatureThresholds: [],
}

const { subscribe: subFans, update: updateFans } = writable([
  defaultFanConfiguration,
])
export const fans = {
  subscribe: subFans,
  edit: (index: number, fan: FanConfiguration): void =>
    updateFans((fans) => {
      fans[index] = fan
      fans = fans
      return fans
    }),
  setLength: (length: number): void =>
    updateFans((fans) => {
      if (length < 1) return fans

      const oldLength = fans.length
      fans = [...fans.slice(0, length)]
      if (length > oldLength) {
        for (let i = 0; i < length - oldLength; i++) {
          fans.push(defaultFanConfiguration)
        }
      // fans.concat(new Array(length -oldLength).fill(defaultFanConfiguration))
      }

      return fans
    }),
}

const { subscribe: fansNumberSub } = derived([fans], (fans) => fans.length)

export const fansNumber = {
  subscribe: fansNumberSub,
  set: (length: number): void => fans.setLength(length),
}
