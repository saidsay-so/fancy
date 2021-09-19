/* eslint-disable import/prefer-default-export */
/* eslint import/no-extraneous-dependencies: ["error", {"devDependencies": true}] */
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { derived, readable } from 'svelte/store';
import type { Subscriber, Unsubscriber } from 'svelte/store';

enum Commands {
    GET_POLL_INTERVAL = 'get_poll_interval',
    GET_TEMPS = 'get_temps',
    GET_SPEEDS = 'get_speeds',
    GET_CONFIG = 'get_config',
    GET_CRITICAL = 'get_critical',
    GET_NAMES = 'get_names',
    GET_TARGET_SPEEDS ='get_target_speeds',
    SET_TARGET_SPEED = 'set_target_speed',
    GET_AUTO = 'get_auto',
    SET_AUTO = 'set_auto',
}

enum Event {
  CONFIG_CHANGE = 'config_change',
  AUTO_CHANGE = 'auto_change',
  TARGET_SPEEDS_CHANGE = 'target_speeds_change'
}

const listenConfig = <T>(cmd: Commands, set: Subscriber<T>) => {
  invoke(cmd).then(set);

  let unlisten: UnlistenFn;
  listen(Event.CONFIG_CHANGE, () => invoke(cmd).then(set))
    .then((un) => { unlisten = un; });
  return unlisten;
};

export const config = readable(null, (set) => {
  invoke(Commands.GET_CONFIG).then(set);

  let unlisten: void | Unsubscriber;
  listen<string>(Event.CONFIG_CHANGE, (ev) => set(ev.payload))
    .then((un) => { unlisten = un; });

  return unlisten;
});

export const pollInterval = readable(1000);

/** Subscribes to the changes for a prop using the poll interval. */
const propSubscriber = <T>(cmd: Commands, set: Subscriber<T>) => {
  const cb = () => {
    invoke(cmd).then(set);
  };

  let intervalCb;
  pollInterval.subscribe((i) => {
    clearInterval(intervalCb);
    intervalCb = setInterval(cb, i);
  });

  return () => {
    clearInterval(intervalCb);
  };
};

export const temperatures = readable({} as Record<string, number>,
  (set) => propSubscriber(Commands.GET_TEMPS, set));

export const meanTemperature = derived(temperatures,
  ($temperatures) => {
    if ($temperatures === null) return null;

    const values = Object.values($temperatures);
    return values.reduce((acc, t) => acc + t, 0) / values.length;
  });

export const fansSpeeds = readable([], (set) => propSubscriber(Commands.GET_SPEEDS, set));

export const critical = readable(false, (set) => propSubscriber(Commands.GET_CRITICAL, set));

export const fansNames = readable([], (set) => listenConfig(Commands.GET_NAMES, set));

export const setTargetSpeed = (index: number, s: number): void => {
  const speed = Math.max(0, Math.min(100, s));

  invoke(Commands.SET_TARGET_SPEED, {
    index,
    speed,
  });
};

export const targetSpeeds = readable([], (set) => {
  let unlisten: UnlistenFn;
  invoke(Commands.GET_TARGET_SPEEDS).then(set);
  listen<unknown[]>(Event.TARGET_SPEEDS_CHANGE, (ev) => set(ev.payload))
    .then((un) => { unlisten = un; });
  return unlisten;
});

const { subscribe: subAuto } = readable(false, (set) => {
  let unlisten: UnlistenFn;
  invoke(Commands.GET_AUTO).then(set);
  listen<boolean>(Event.AUTO_CHANGE, (ev) => set(ev.payload))
    .then((un) => { unlisten = un; });
  return unlisten;
});

export const auto = {
  subscribe: subAuto,
  set: (auto: boolean): void => {
    invoke(Commands.SET_AUTO, { auto });
  },
};
