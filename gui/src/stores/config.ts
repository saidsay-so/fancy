import { invoke } from '@tauri-apps/api'
import { derived, readable, writable } from 'svelte/store'

enum Commands {
  GET_CONFIGS_LIST = 'get_configs_list',
  GET_MODEL = 'get_model',
  SET_CONFIG = 'set_config',
}

export interface Threshold {
  UpThreshold: number
  DownThreshold: number
  FanSpeed: number
}

export interface ConfigInfo {
  author: string | null
  model: string
  name: string
  thresholds: Record<string, Threshold[]>
}

export const model = readable<string>('', (set) => {
  invoke(Commands.GET_MODEL).then(set)
})

export const activeDetails = writable<ConfigInfo>({} as ConfigInfo)

export const getConfigsList = readable<ConfigInfo[]>([], (set) => {
  invoke(Commands.GET_CONFIGS_LIST).then(set)
})

export const filteredConfigsList = derived(
  [getConfigsList, model],
  ([$configs, $model]) =>
    $configs
      .map((config) => {
        const name = config.name
        const nameParts = name.toLowerCase().split(' ')
        const modelParts = $model.toLowerCase().split(' ')
        let similarity = 0

        for (const modelPart of modelParts) {
          similarity += nameParts
            .map((namePart) => {
              let commonLetters = 0
              const len = Math.min(namePart.length, modelPart.length)
              while (
                commonLetters < len &&
                namePart[commonLetters] === modelPart[commonLetters]
              ) {
                commonLetters++
              }
              return commonLetters / Math.max(namePart.length, modelPart.length)
            })
            .reduce(
              (maxSimilarity, similarity) =>
                Math.max(maxSimilarity, similarity),
              0
            )
        }

        return { ...config, similarity }
      })
      .filter(({ similarity }) => similarity > 2)
      .sort(({ similarity: a }, { similarity: b }) => b - a)
      .map(({ similarity, ...config }) => config as ConfigInfo)
)

export const setConfig = (config: string): Promise<void> =>
  invoke(Commands.SET_CONFIG, { config })
