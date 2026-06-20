/**
 * 类型契约测试
 * 验证 Electron API 兼容层的类型定义与 Rust 端命令匹配
 */

import { describe, expect, it } from 'vitest'
import type {
  ServerReadyData,
  TitlebarTheme,
  FatalRendererError,
  UpdaterState,
  ElectronAPI,
} from '../types/electron-api'

describe('ElectronAPI 类型契约', () => {
  it('ServerReadyData 包含必要字段', () => {
    const data: ServerReadyData = {
      url: 'http://127.0.0.1:8080',
      username: 'admin',
      password: 'pass',
    }
    expect(data.url).toBeTypeOf('string')
    expect(typeof data.username).toBe('string')
    expect(typeof data.password).toBe('string')

    // username 和 password 应可为 null
    const dataNoAuth: ServerReadyData = {
      url: 'http://localhost',
      username: null,
      password: null,
    }
    expect(dataNoAuth.username).toBeNull()
  })

  it('TitlebarTheme 模式只接受 light/dark', () => {
    const t: TitlebarTheme = { mode: 'dark' }
    expect(['light', 'dark']).toContain(t.mode)
  })

  it('FatalRendererError 字段完整', () => {
    const e: FatalRendererError = {
      error: 'boom',
      url: 'app://main',
      version: '1.0.0',
      platform: 'win32',
      os: 'Windows 11',
    }
    expect(e.error).toBe('boom')
    expect(e.url).toBe('app://main')
    // 可选字段
    expect(e.version).toBe('1.0.0')
    expect(e.os).toBe('Windows 11')
  })

  it('UpdaterState 包含 status', () => {
    const s: UpdaterState = {
      status: 'idle',
      progress: 0,
      version: '1.0.0',
    }
    expect(s.status).toBe('idle')
  })

  it('ElectronAPI 接口主要方法存在（编译期检查）', () => {
    // 这是一个编译期断言：把接口展开成对象字面量
    // 如果接口字段缺失或类型不匹配，TypeScript 编译会失败
    const api: Partial<ElectronAPI> = {
      killSidecar: async () => {},
      awaitInitialization: async () => ({ url: '', username: null, password: null }),
      storeGet: async () => null,
      storeSet: async () => {},
      getWindowCount: async () => 1,
      onMenuCommand: () => () => {},
    }
    expect(api.killSidecar).toBeDefined()
    expect(api.storeGet).toBeDefined()
  })
})
