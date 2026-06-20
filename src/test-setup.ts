// Vitest 全局测试设置
// Mock Tauri 运行时，避免真实调用 Tauri 桥

import { vi } from 'vitest'

// Mock @tauri-apps/api/core 的 invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: unknown) => {
    // 简单回显：让测试可以断言调用参数
    return { cmd, args }
  }),
}))

// Mock @tauri-apps/plugin-shell
vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn(async () => {}),
}))

// Mock @tauri-apps/plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(async () => null),
  save: vi.fn(async () => null),
  message: vi.fn(async () => undefined),
  ask: vi.fn(async () => false),
  confirm: vi.fn(async () => false),
}))

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
  emit: vi.fn(async () => {}),
}))

// Mock @tauri-apps/api/window
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    listen: vi.fn(async () => () => {}),
    close: vi.fn(),
    minimize: vi.fn(),
    maximize: vi.fn(),
    unmaximize: vi.fn(),
    toggleMaximize: vi.fn(),
    isMaximized: vi.fn(async () => false),
  })),
}))
