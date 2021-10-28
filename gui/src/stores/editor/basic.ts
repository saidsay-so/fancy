import { writable } from 'svelte/store'
import type { FanControlConfigV2 } from 'nbfc-types'
import { model as computerModel } from '../config'

export const configName = writable<string>('', (set) =>
  computerModel.subscribe(set)
)
export const model = writable<string>('', (set) => computerModel.subscribe(set))
export const author = writable<string>('')
export const ecPollInterval = writable<number>(1000)
export const readWriteWords = writable<boolean>(false)
export const criticalTemp = writable<number>(50)
