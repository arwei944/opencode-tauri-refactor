/// <reference types="vitest" />
import { defineConfig } from 'vitest/config'
import solid from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solid()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    // 避免 Tauri 模块的真实导入报错
    server: {
      deps: {
        inline: ['@tauri-apps/api', '@tauri-apps/plugin-shell', '@tauri-apps/plugin-dialog'],
      },
    },
  },
})
