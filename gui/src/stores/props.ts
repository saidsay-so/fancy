import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri'
import { derived, readable, writable } from 'svelte/store'
import type { Subscriber } from 'svelte/store'
import { errors } from './error'
import type { BackendError } from './error'

enum PropCommand {
  GET_POLL_INTERVAL = 'get_poll_interval',
  GET_TEMPS = 'get_temps',
  GET_SPEEDS = 'get_speeds',
  GET_CONFIG = 'get_config',
  SET_CONFIG = 'set_config',
  GET_CRITICAL = 'get_critical',
  GET_NAMES = 'get_names',
  GET_TARGET_SPEEDS = 'get_target_speeds',
  SET_TARGET_SPEED = 'set_target_speed',
  GET_AUTO = 'get_auto',
  SET_AUTO = 'set_auto',
}

enum Event {
  CONFIG_CHANGE = 'config_change',
  AUTO_CHANGE = 'auto_change',
  TARGET_SPEEDS_CHANGE = 'target_speeds_change',
}

const catchError = ({ message, name, critical }: BackendError) =>
  errors.report(message, critical, name)

const listenEvent = <T>(
  cmd: PropCommand,
  event: Event,
  set: Subscriber<T>,
  listener: (value: T) => void = set
) => {
  invoke<T>(cmd).then(set).catch(catchError)

  let unlisten: UnlistenFn
  listen<T>(event, (ev) => listener(ev.payload)).then((un) => {
    unlisten = un
  })
  return unlisten
}

export const config = readable(null, (set) =>
  listenEvent(PropCommand.GET_CONFIG, Event.CONFIG_CHANGE, set)
)

export const pollInterval = readable(Infinity, (set) =>
  listenEvent(PropCommand.GET_POLL_INTERVAL, Event.CONFIG_CHANGE, set, () =>
    invoke(PropCommand.GET_POLL_INTERVAL).then(set).catch(catchError)
  )
)

/** Poll a prop using the poll interval. */
const propSubscriber = <T>(cmd: PropCommand, set: Subscriber<T>) => {
  const cb = () => {
    invoke(cmd).then(set).catch(catchError)
  }

  let intervalCb: ReturnType<typeof setTimeout>
  pollInterval.subscribe((i) => {
    clearInterval(intervalCb)
    if (i !== Infinity) intervalCb = setInterval(cb, i)
  })

  return () => {
    clearInterval(intervalCb)
  }
}

export const temperatures = readable({} as Record<string, number>, (set) =>
  propSubscriber(PropCommand.GET_TEMPS, set)
)

export const meanTemperature = derived(temperatures, ($temperatures) => {
  const values = Object.values($temperatures)
  return values.reduce((acc, t) => acc + t, 0) / values.length
})

export const fansSpeeds = readable<number[]>([], (set) =>
  propSubscriber(PropCommand.GET_SPEEDS, set)
)

export const critical = readable(false, (set) =>
  propSubscriber(PropCommand.GET_CRITICAL, set)
)

export const fansNames = readable<string[]>([], (set) =>
  listenEvent(PropCommand.GET_NAMES, Event.CONFIG_CHANGE, set, () =>
    invoke(PropCommand.GET_NAMES).then(set).catch(catchError)
  )
)

export const targetSpeeds = writable<number[]>([], (set) =>
  listenEvent(PropCommand.GET_TARGET_SPEEDS, Event.TARGET_SPEEDS_CHANGE, set)
)

export const setTargetSpeed = (index: number, s: number): void => {
  const speed = Math.max(0, Math.min(100, s))

  invoke(PropCommand.SET_TARGET_SPEED, { speed, index }).catch(catchError)
}

const { subscribe: subAuto } = readable(undefined, (set) =>
  listenEvent(PropCommand.GET_AUTO, Event.AUTO_CHANGE, set)
)

export const auto = {
  subscribe: subAuto,
  set: (auto: boolean): void => {
    if (auto !== undefined)
      invoke(PropCommand.SET_AUTO, { auto }).catch(catchError)
  },
}
