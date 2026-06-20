/**
 * useElectronApi Hook 单元测试
 * 验证兼容层正确调用 Tauri 命令
 */

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import { createRoot } from 'solid-js'
import { useElectronApi } from './use-electron-api'
import { invoke } from '@tauri-apps/api/core'
import { open as shellOpen } from '@tauri-apps/plugin-shell'
import { open as dialogOpen, save as dialogSave } from '@tauri-apps/plugin-dialog'

// 获取 mock 的 invoke
const mockInvoke = vi.mocked(invoke)
const mockShellOpen = vi.mocked(shellOpen)
const mockDialogOpen = vi.mocked(dialogOpen)
const mockDialogSave = vi.mocked(dialogSave)

describe('useElectronApi Hook', () => {
  beforeEach(() => {
    mockInvoke.mockClear()
    mockShellOpen.mockClear()
    mockDialogOpen.mockClear()
    mockDialogSave.mockClear()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  // 工具函数：在 reactive root 中创建 hook（用完即销毁）
  function withHook<T>(fn: (api: ReturnType<typeof useElectronApi>) => T): T {
    return createRoot((dispose) => {
      const result = fn(useElectronApi())
      dispose()
      return result
    })
  }

  it('返回 isReady=true', () => {
    withHook(({ isReady }) => {
      expect(isReady).toBe(true)
    })
  })

  it('返回包含 api 对象', () => {
    withHook(({ api }) => {
      expect(api).toBeDefined()
    })
  })

  // ---- 简单 invoke 命令 ----

  it('killSidecar 调用 invoke("kill_sidecar")', async () => {
    await withHook(async ({ api }) => {
      await api.killSidecar?.()
      expect(mockInvoke).toHaveBeenCalledWith('kill_sidecar')
    })
  })

  it('awaitInitialization 调用 invoke("await_initialization")', async () => {
    await withHook(async ({ api }) => {
      await api.awaitInitialization?.()
      expect(mockInvoke).toHaveBeenCalledWith('await_initialization')
    })
  })

  it('getDefaultServerUrl 调用 invoke("get_default_server_url")', async () => {
    await withHook(async ({ api }) => {
      await api.getDefaultServerUrl?.()
      expect(mockInvoke).toHaveBeenCalledWith('get_default_server_url')
    })
  })

  it('setDefaultServerUrl 传递 url 参数', async () => {
    await withHook(async ({ api }) => {
      await api.setDefaultServerUrl?.('http://example.com')
      expect(mockInvoke).toHaveBeenCalledWith('set_default_server_url', { url: 'http://example.com' })

      await api.setDefaultServerUrl?.(null)
      expect(mockInvoke).toHaveBeenCalledWith('set_default_server_url', { url: null })
    })
  })

  // ---- Store 操作 ----

  it('storeGet 传递 name 和 key', async () => {
    await withHook(async ({ api }) => {
      await api.storeGet?.('settings', 'theme')
      expect(mockInvoke).toHaveBeenCalledWith('store_get', { name: 'settings', key: 'theme' })
    })
  })

  it('storeSet 传递 name、key、value', async () => {
    await withHook(async ({ api }) => {
      await api.storeSet?.('settings', 'theme', 'dark')
      expect(mockInvoke).toHaveBeenCalledWith('store_set', {
        name: 'settings',
        key: 'theme',
        value: 'dark',
      })
    })
  })

  it('storeDelete 传递 name 和 key', async () => {
    await withHook(async ({ api }) => {
      await api.storeDelete?.('settings', 'theme')
      expect(mockInvoke).toHaveBeenCalledWith('store_delete', { name: 'settings', key: 'theme' })
    })
  })

  it('storeClear 只传递 name', async () => {
    await withHook(async ({ api }) => {
      await api.storeClear?.('settings')
      expect(mockInvoke).toHaveBeenCalledWith('store_clear', { name: 'settings' })
    })
  })

  // ---- 窗口操作 ----

  it('窗口方法调用正确命令', async () => {
    await withHook(async ({ api }) => {
      await api.getWindowCount?.()
      expect(mockInvoke).toHaveBeenCalledWith('get_window_count')

      await api.setWindowFocus?.()
      expect(mockInvoke).toHaveBeenCalledWith('set_window_focus')

      await api.showWindow?.()
      expect(mockInvoke).toHaveBeenCalledWith('show_window')

      await api.relaunch?.()
      expect(mockInvoke).toHaveBeenCalledWith('relaunch')
    })
  })

  // ---- 缩放 ----

  it('setZoomFactor 传递 factor 参数', async () => {
    await withHook(async ({ api }) => {
      await api.setZoomFactor?.(1.5)
      expect(mockInvoke).toHaveBeenCalledWith('set_zoom_factor', { factor: 1.5 })
    })
  })

  it('setPinchZoomEnabled 传递布尔参数', async () => {
    await withHook(async ({ api }) => {
      await api.setPinchZoomEnabled?.(true)
      expect(mockInvoke).toHaveBeenCalledWith('set_pinch_zoom_enabled', { enabled: true })
    })
  })

  // ---- Shell 操作 ----

  it('openLink 通过 plugin-shell.open 打开 URL', async () => {
    await withHook(async ({ api }) => {
      await api.openLink?.('https://opencode.ai')
      expect(mockShellOpen).toHaveBeenCalledWith('https://opencode.ai')
    })
  })

  it('openPath 不带 app 时走 plugin-shell.open', async () => {
    await withHook(async ({ api }) => {
      await api.openPath?.('/tmp/file.txt')
      expect(mockShellOpen).toHaveBeenCalledWith('/tmp/file.txt')
    })
  })

  it('openPath 带 app 时走 invoke("open_path")', async () => {
    await withHook(async ({ api }) => {
      await api.openPath?.('/tmp/file.txt', 'code')
      expect(mockInvoke).toHaveBeenCalledWith('open_path', {
        path: '/tmp/file.txt',
        app: 'code',
      })
    })
  })

  // ---- 文件对话框 ----

  it('saveFilePicker 走 plugin-dialog.save', async () => {
    await withHook(async ({ api }) => {
      const r = await api.saveFilePicker?.({ title: '保存为' })
      expect(mockDialogSave).toHaveBeenCalledWith(
        expect.objectContaining({ title: '保存为' })
      )
      expect(r).toBeNull() // mock 返回 null
    })
  })

  // ---- 工具方法 ----

  it('getPathForFile 优先返回 file.path', () => {
    withHook(({ api }) => {
      const fakeFile = { path: '/some/path.txt', name: 'path.txt' } as unknown as File
      expect(api.getPathForFile?.(fakeFile)).toBe('/some/path.txt')
    })
  })

  it('getPathForFile 回退到 file.name', () => {
    withHook(({ api }) => {
      const fakeFile = { name: 'fallback.txt' } as unknown as File
      expect(api.getPathForFile?.(fakeFile)).toBe('fallback.txt')
    })
  })

  // ---- 事件订阅 ----

  it('onMenuCommand 返回取消订阅函数', () => {
    withHook(({ api }) => {
      const cb = vi.fn()
      const unsub = api.onMenuCommand?.(cb)
      expect(typeof unsub).toBe('function')
      // 多次调用不应抛错
      unsub?.()
    })
  })

  it('onDeepLink 返回取消订阅函数', () => {
    withHook(({ api }) => {
      const cb = vi.fn()
      const unsub = api.onDeepLink?.(cb)
      expect(typeof unsub).toBe('function')
      unsub?.()
    })
  })

  // ---- 通知 ----

  it('showNotification 传递 title 和 body', async () => {
    await withHook(async ({ api }) => {
      await api.showNotification?.('Hello', 'World')
      expect(mockInvoke).toHaveBeenCalledWith('show_notification', {
        title: 'Hello',
        body: 'World',
      })
    })
  })

  // ---- 错误记录 ----

  it('recordFatalRendererError 传递 error 对象', async () => {
    await withHook(async ({ api }) => {
      await api.recordFatalRendererError?.({
        error: 'oops',
        url: 'app://main',
        platform: 'test',
      })
      expect(mockInvoke).toHaveBeenCalledWith(
        'record_fatal_renderer_error',
        expect.objectContaining({
          error: expect.objectContaining({ error: 'oops' }),
        })
      )
    })
  })

  // ---- WSL/Updater stub ----

  it('wslServers.update 暴露所有 WSL 方法', () => {
    withHook(({ api }) => {
      expect(api.wslServers?.getState).toBeDefined()
      expect(api.wslServers?.refreshDistros).toBeDefined()
      expect(api.wslServers?.addServer).toBeDefined()
    })
  })

  it('updater.check 返回 idle 状态（stub）', async () => {
    await withHook(async ({ api }) => {
      const state = await api.updater?.check()
      expect(state).toEqual({ status: 'idle' })
    })
  })
})
