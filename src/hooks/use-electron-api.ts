/**
 * Hook to provide Electron-like API using Tauri commands
 * This allows the existing OpenCode codebase to work with minimal changes
 */

import { createSignal, onCleanup } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { save, open as openDialog } from '@tauri-apps/plugin-dialog'
import type {
  ServerReadyData,
  TitlebarTheme,
  FatalRendererError,
  ElectronAPI,
} from '../types/electron-api'

export interface UseElectronApiOptions {
  // Options for the hook
}

export interface UseElectronApiReturn {
  api: Partial<ElectronAPI>
  isReady: boolean
}

export function useElectronApi(_options?: UseElectronApiOptions): UseElectronApiReturn {
  const [isReady, setIsReady] = createSignal(false)

  // 订阅状态（为未来功能预留）
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_updaterState, _setUpdaterState] = createSignal<any>(null)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_wslServersState, _setWslServersState] = createSignal<any>(null)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_menuCommandHandler, setMenuCommandHandler] = createSignal<((id: string) => void) | null>(null)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_deepLinkHandler, setDeepLinkHandler] = createSignal<((urls: string[]) => void) | null>(null)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_pinchZoomHandler, setPinchZoomHandler] = createSignal<((enabled: boolean) => void) | null>(null)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_zoomFactorHandler, setZoomFactorHandler] = createSignal<((factor: number) => void) | null>(null)

  // Implement the API
  const api: Partial<ElectronAPI> = {
    // Sidecar management
    killSidecar: async () => {
      return invoke('kill_sidecar')
    },

    installCli: async () => {
      // TODO: Implement CLI installation
      return 'CLI installed'
    },

    // Initialization
    awaitInitialization: async () => {
      const data = await invoke('await_initialization') as ServerReadyData
      return data
    },

    // Server URL
    getDefaultServerUrl: async () => {
      return (await invoke('get_default_server_url')) as string | null
    },

    setDefaultServerUrl: async (url: string | null) => {
      return invoke('set_default_server_url', { url })
    },

    // Display backend
    getDisplayBackend: async () => {
      return (await invoke('get_display_backend')) as any
    },

    setDisplayBackend: async (backend: any) => {
      return invoke('set_display_backend', { backend })
    },

    // Markdown
    parseMarkdownCommand: async (markdown: string) => {
      return invoke('parse_markdown', { markdown }) as Promise<string>
    },

    // App checking
    checkAppExists: async (appName: string) => {
      return invoke('check_app_exists', { appName }) as Promise<boolean>
    },

    resolveAppPath: async (appName: string) => {
      return invoke('resolve_app_path', { appName }) as Promise<string | null>
    },

    // Store operations
    storeGet: async (name: string, key: string) => {
      return invoke('store_get', { name, key }) as Promise<string | null>
    },

    storeSet: async (name: string, key: string, value: string) => {
      return invoke('store_set', { name, key, value })
    },

    storeDelete: async (name: string, key: string) => {
      return invoke('store_delete', { name, key })
    },

    storeClear: async (name: string) => {
      return invoke('store_clear', { name })
    },

    storeKeys: async (name: string) => {
      return invoke('store_keys', { name }) as Promise<string[]>
    },

    storeLength: async (name: string) => {
      return invoke('store_length', { name }) as Promise<number>
    },

    // Window operations
    getWindowCount: async () => {
      return invoke('get_window_count') as Promise<number>
    },

    getWindowFocused: async () => {
      return invoke('get_window_focused') as Promise<boolean>
    },

    setWindowFocus: async () => {
      return invoke('set_window_focus')
    },

    showWindow: async () => {
      return invoke('show_window')
    },

    // Zoom
    getZoomFactor: async () => {
      return invoke('get_zoom_factor') as Promise<number>
    },

    setZoomFactor: async (factor: number) => {
      return invoke('set_zoom_factor', { factor })
    },

    getPinchZoomEnabled: async () => {
      return invoke('get_pinch_zoom_enabled') as Promise<boolean>
    },

    setPinchZoomEnabled: async (enabled: boolean) => {
      return invoke('set_pinch_zoom_enabled', { enabled })
    },

    // Subscriptions
    onMenuCommand: (cb: (id: string) => void) => {
      setMenuCommandHandler(() => cb)
      // In Tauri, we would use event listeners
      // For now, just store the callback
      return () => setMenuCommandHandler(null)
    },

    onDeepLink: (cb: (urls: string[]) => void) => {
      setDeepLinkHandler(() => cb)
      return () => setDeepLinkHandler(null)
    },

    onPinchZoomEnabledChanged: (cb: (enabled: boolean) => void) => {
      setPinchZoomHandler(() => cb)
      return () => setPinchZoomHandler(null)
    },

    onZoomFactorChanged: (cb: (factor: number) => void) => {
      setZoomFactorHandler(() => cb)
      return () => setZoomFactorHandler(null)
    },

    // File operations
    openDirectoryPicker: async (opts?: { multiple?: boolean; title?: string; defaultPath?: string }) => {
      // Tauri dialog doesn't support multiple directory selection the same way
      // This is a simplified implementation
      if (opts?.multiple) {
        // For multiple selection, we might need to open dialog multiple times
        // or use a different approach
        const result = await openDialog({
          directory: true,
          multiple: false,
          title: opts?.title || 'Select Directory',
          defaultPath: opts?.defaultPath,
        })
        return result ? [result] : null
      }
      
      const result = await openDialog({
        directory: true,
        multiple: false,
        title: opts?.title || 'Select Directory',
        defaultPath: opts?.defaultPath,
      })
      return result || null
    },

    openFilePicker: async (opts?: { multiple?: boolean; title?: string; defaultPath?: string; extensions?: string[] }) => {
      if (opts?.multiple) {
        const result = await openDialog({
          multiple: true,
          title: opts?.title || 'Select Files',
          defaultPath: opts?.defaultPath,
          filters: opts?.extensions ? [{
            name: 'Files',
            extensions: opts.extensions,
          }] : undefined,
        })
        
        if (result && Array.isArray(result)) {
          // Generate token (simplified)
          const token = Math.random().toString(36).substring(2)
          const files = await Promise.all(
            (result as string[]).map(async (path) => {
              // In a real implementation, we would get file info
              return { path, name: path.split('/').pop() || path, size: 0 }
            })
          )
          return { token, files }
        }
        return null
      }

      const result = await openDialog({
        multiple: false,
        title: opts?.title || 'Select File',
        defaultPath: opts?.defaultPath,
        filters: opts?.extensions ? [{
          name: 'Files',
          extensions: opts.extensions,
        }] : undefined,
      })

      if (result) {
        const token = Math.random().toString(36).substring(2)
        const files = [{
          path: result as string,
          name: (result as string).split('/').pop() || result,
          size: 0,
        }]
        return { token, files }
      }
      return null
    },

    saveFilePicker: async (opts?: { title?: string; defaultPath?: string }) => {
      const result = await save({
        title: opts?.title || 'Save File',
        defaultPath: opts?.defaultPath,
      })
      return result || null
    },

    // Other operations
    openLink: async (url: string) => {
      return open(url)
    },

    openPath: async (path: string, app?: string) => {
      if (app) {
        // Open with specific app
        return invoke('open_path', { path, app })
      }
      return open(path)
    },

    showNotification: async (title: string, body?: string) => {
      return invoke('show_notification', { title, body })
    },

    relaunch: async () => {
      return invoke('relaunch')
    },

    setBackgroundColor: async (color: string) => {
      return invoke('set_background_color', { color })
    },

    exportDebugLogs: async () => {
      return invoke('export_debug_logs') as Promise<string>
    },

    recordFatalRendererError: async (error: FatalRendererError) => {
      return invoke('record_fatal_renderer_error', { error })
    },

    setTitlebar: async (theme: TitlebarTheme) => {
      return invoke('set_titlebar', { theme })
    },

    runDesktopMenuAction: async (action: string) => {
      return invoke('run_desktop_menu_action', { action })
    },

    consumeInitialDeepLinks: async () => {
      return invoke('consume_initial_deep_links') as Promise<string[]>
    },

    // WSL Servers (stub implementation)
    wslServers: {
      getState: async () => {
        return {}
      },
      subscribe: async (_cb: (event: any) => void) => {
        // Stub
        return () => {}
      },
      probeRuntime: async () => {
        return {}
      },
      refreshDistros: async () => {},
      installWsl: async () => {},
      installDistro: async () => {},
      probeDistro: async () => {
        return {}
      },
      probeOpencode: async () => {
        return {}
      },
      installOpencode: async () => {},
      openTerminal: async () => {},
      addServer: async () => {},
      removeServer: async () => {},
      startServer: async () => {},
    },

    // Updater (stub implementation)
    updater: {
      subscribe: async (_cb: (state: any) => void) => {
        // Stub - in real implementation, we would use Tauri's updater or custom solution
        return () => {}
      },
      check: async () => {
        return { status: 'idle' }
      },
      install: async () => {},
    },

    // getPathForFile implementation
    getPathForFile: (file: File) => {
      // Tauri 没有直接的等价物，用类型断言获取可能存在的 path 属性
      // 在 Electron 中 File 对象有 path 属性，Tauri 中需要通过对话框获取
      return (file as any).path || file.name
    },

    // readPickedFile and releasePickedFiles
    readPickedFile: async (_token: string, _path: string) => {
      // In Tauri, we would read the file directly
      throw new Error('readPickedFile not implemented in Tauri version')
    },

    releasePickedFiles: async (_token: string) => {
      // In Tauri, files are managed differently
      // This is a no-op for now
    },
  }

  // Mark as ready
  // In a real implementation, we would wait for initialization
  // For now, we'll just set it to true immediately
  setIsReady(true)

  // Cleanup subscriptions
  onCleanup(() => {
    setMenuCommandHandler(null)
    setDeepLinkHandler(null)
    setPinchZoomHandler(null)
    setZoomFactorHandler(null)
  })

  return { api, isReady: isReady() }
}

export default useElectronApi
