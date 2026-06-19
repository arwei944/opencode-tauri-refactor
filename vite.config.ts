import { defineConfig } from 'vite'
import solidPlugin from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
    rollupOptions: {
      external: ['@tauri-apps/api', '@tauri-apps/api/core', '@tauri-apps/api/shell']
    }
  },
  clearScreen: false,
})
