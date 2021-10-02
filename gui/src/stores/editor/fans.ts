import { derived, writable } from 'svelte/store'

export const fansNumber = writable(1)
export const fans = writable([])
export const fan = (index: number) => derived([fans], ($fans) => $fans[index])
