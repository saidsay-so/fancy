import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { readable } from 'svelte/store';

enum ErrorEvent {
  CONNECTION_ERROR = 'connection_error'
}

export const connectionError = readable(null, (set) => {
  let unlisten: UnlistenFn;
  listen(ErrorEvent.CONNECTION_ERROR, set).then((un) => { unlisten = un; });
  return unlisten;
});
