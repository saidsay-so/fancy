/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api";
import { derived, readable, writable } from "svelte/store";

enum Commands {
  GET_CONFIGS_LIST = "get_configs_list",
  GET_MODEL = "get_model",
  SET_CONFIG = "set_config",
}

export interface Threshold {
  UpThreshold: number;
  DownThreshold: number;
  FanSpeed: number;
}

interface ConfigInfo {
  author: string | null;
  name: string;
  path: string;
  thresholds: Record<string, Threshold[]>;
}

export const model = readable<string>("", (set) => {
  invoke(Commands.GET_MODEL).then(set);
});

export const activeDetails = writable<ConfigInfo>({} as ConfigInfo);

export const getConfigsList = readable<ConfigInfo[]>([], (set) => {
  invoke(Commands.GET_CONFIGS_LIST).then(set);
});

export const filteredConfigsList = derived(
  [getConfigsList, model],
  ([$configs, model]) =>
    $configs.filter((config) => {
      const name = config.name.length > 0 ? config.name : config.path;
      const nameParts = name.split(" ");
      let indice = 0;

      for (const modelPart of model.split(" ")) {
        indice += nameParts
          .map((namePart) => {
            let i = 0;
            for (; i < Math.min(namePart.length, modelPart.length); i++) {
              if (name[i] !== model[i]) {
                break;
              }
            }
            return i;
          })
          .reduce(
            (maxSimilarity, similarity) => Math.max(maxSimilarity, similarity),
            0
          );
      }

      return indice > 5;
    })
);

export const setConfig = (config: string): Promise<void> =>
  invoke(Commands.SET_CONFIG, { config });
