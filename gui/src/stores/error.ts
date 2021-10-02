import { listen } from '@tauri-apps/api/event'
import { derived, writable } from 'svelte/store'

enum ErrorEvent {
  CONNECTION_ERROR = 'connection_error',
  PROXY_ERROR = 'proxy_error',
}

export class AppError extends Error {
  timestamp: number
  fatal: boolean

  constructor(
    name: string,
    message: string,
    fatal: boolean,
    timestamp: number
  ) {
    super(message)
    super.name = name
    this.timestamp = timestamp
    this.fatal = fatal
  }

  toString(): string {
    return `[${new Date(this.timestamp).toLocaleTimeString()}]: ${
      this.name
    } | ${this.fatal ? 'Fatal Error |' : ''} ${this.message}}`
  }
}

export interface BackendError {
  critical: boolean
  name: string
  message: string
}

const { subscribe: subErrors, update: updateErrors } = writable<AppError[]>(
  [],
  () => {
    for (const ev of Object.values(ErrorEvent)) {
      listen<BackendError>(ev, (ev) => {
        const { message, name, critical } = ev.payload
        console.error(message)
        errors.report(message, critical, name)
      })
    }
  }
)

export const errors = {
  subscribe: subErrors,
  report: (
    message: string,
    fatal = true,
    name = 'AppError',
    timestamp = Date.now()
  ): void =>
    updateErrors((errors) => [
      ...errors,
      new AppError(name, message, fatal, timestamp),
    ]),
}

export const lastError = derived(errors, ($errors) =>
  $errors.length > 1 ? $errors[$errors.length - 1] : undefined
)

lastError.subscribe((error) => {
  if (error !== undefined) console.error(error)
})
