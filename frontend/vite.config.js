import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'
import wasm from 'vite-plugin-wasm'
// https://vite.dev/config/
export default defineConfig({
  base: '/Rustarcade/',
  plugins: [
    svelte(),
    tailwindcss(),
    wasm(),
  ],
  server: {
    fs: {
      allow: ['..'],
    },
  },
})
