import { defineConfig } from 'vite'
import { resolve } from 'path'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import analyze from 'rollup-plugin-analyzer'

// https://vitejs.dev/config/
export default defineConfig({
    build: {
    rollupOptions: {
      plugins: [analyze()],
    },
  },
  plugins: [svelte()],
  resolve: { 
    alias: {
      '~': resolve(__dirname, './src'),
      '@': resolve(__dirname, './src'),
    }
  }
})
