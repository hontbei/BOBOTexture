import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  build: {
    emptyOutDir: false,
  },
  server: {
    port: 1420,
    strictPort: true,
    host: '0.0.0.0',
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
})
