import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  base: './',
  plugins: [vue()],
  build: {
    outDir: 'dist'
  },
  server: {
    proxy: {
      '/ws': {
        target: 'ws://127.0.0.1:8999',
        ws: true
      }
    }
  }
})
