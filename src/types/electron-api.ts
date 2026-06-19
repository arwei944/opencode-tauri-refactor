/**
 * Electron API compatibility types for Tauri
 * This file provides type definitions that match the original Electron desktop API
 * to make migration easier.
 */

export interface ServerReadyData {
  url: string
  username: string | null
  password: string | null
}

export interface TitlebarTheme {
  mode: 'light' | 'dark'
}

export interface FatalRendererError {
  error: string
  url: string
  version?: string
  platform: string
  os?: string
}

export interface WslDistroProbe {
  // WSL distro probe type
}

export interface WslInstalledDistro {
  // WSL installed distro type
}

export interface WslJob {
  // WSL job type
}

export interface WslOnlineDistro {
  // WSL online distro type
}

export interface WslOpencodeCheck {
  // WSL opencode check type
}

export interface WslRuntimeCheck {
  // WSL runtime check type
}

export interface WslServerConfig {
  // WSL server config type
}

export interface WslServerItem {
  // WSL server item type
}

export interface WslServerRuntime {
  // WSL server runtime type
}

export interface WslServersEvent {
  // WSL servers event type
}

export interface WslServersState {
  // WSL servers state type
}

export interface WslServersPlatform {
  getState: () => Promise<WslServersState>
  subscribe: (cb: (event: WslServersEvent) => void) => Promise<() => void>
  probeRuntime: () => Promise<WslRuntimeCheck>
  refreshDistros: () => Promise<void>
  installWsl: () => Promise<void>
  installDistro: (name: string) => Promise<void>
  probeDistro: (name: string) => Promise<WslDistroProbe>
  probeOpencode: (name: string) => Promise<WslOpencodeCheck>
  installOpencode: (name: string) => Promise<void>
  openTerminal: (name: string) => Promise<void>
  addServer: (distro: any) => Promise<void>
  removeServer: (id: string) => Promise<void>
  startServer: (id: string) => Promise<void>
}

export interface UpdaterState {
  // Updater state type
  status: string
  progress?: number
  version?: string
  downloaded?: number
  total?: number
}

export interface UpdaterAPI {
  subscribe: (cb: (state: UpdaterState) => void) => Promise<() => void>
  check: () => Promise<UpdaterState>
  install: () => Promise<void>
}

export type LinuxDisplayBackend = 'wayland' | 'auto'

export type DesktopMenuAction = string // This would be more specific in the real implementation

// Main Electron API interface
export interface ElectronAPI {
  killSidecar: () => Promise<void>
  installCli: () => Promise<string>
  awaitInitialization: () => Promise<ServerReadyData>
  wslServers: WslServersPlatform
  updater: UpdaterAPI
  consumeInitialDeepLinks: () => Promise<string[]>
  getDefaultServerUrl: () => Promise<string | null>
  setDefaultServerUrl: (url: string | null) => Promise<void>
  getDisplayBackend: () => Promise<LinuxDisplayBackend | null>
  setDisplayBackend: (backend: LinuxDisplayBackend | null) => Promise<void>
  parseMarkdownCommand: (markdown: string) => Promise<string>
  checkAppExists: (appName: string) => Promise<boolean>
  resolveAppPath: (appName: string) => Promise<string | null>
  storeGet: (name: string, key: string) => Promise<string | null>
  storeSet: (name: string, key: string, value: string) => Promise<void>
  storeDelete: (name: string, key: string) => Promise<void>
  storeClear: (name: string) => Promise<void>
  storeKeys: (name: string) => Promise<string[]>
  storeLength: (name: string) => Promise<number>

  getWindowCount: () => Promise<number>
  onMenuCommand: (cb: (id: string) => void) => () => void
  onDeepLink: (cb: (urls: string[]) => void) => () => void

  openDirectoryPicker: (opts?: {
    multiple?: boolean
    title?: string
    defaultPath?: string
  }) => Promise<string | string[] | null>
  openFilePicker: (opts?: {
    multiple?: boolean
    title?: string
    defaultPath?: string
    extensions?: string[]
  }) => Promise<{ token: string; files: { path: string; name: string; size: number }[] } | null>
  readPickedFile: (token: string, path: string) => Promise<ArrayBuffer>
  releasePickedFiles: (token: string) => Promise<void>
  getPathForFile: (file: File) => string
  saveFilePicker: (opts?: { title?: string; defaultPath?: string }) => Promise<string | null>
  openLink: (url: string) => void
  openPath: (path: string, app?: string) => Promise<void>
  readClipboardImage: () => Promise<{ buffer: ArrayBuffer; width: number; height: number } | null>
  showNotification: (title: string, body?: string) => void
  getWindowFocused: () => Promise<boolean>
  setWindowFocus: () => Promise<void>
  showWindow: () => Promise<void>
  relaunch: () => void
  getZoomFactor: () => Promise<number>
  setZoomFactor: (factor: number) => Promise<void>
  getPinchZoomEnabled: () => Promise<boolean>
  setPinchZoomEnabled: (enabled: boolean) => Promise<void>
  onPinchZoomEnabledChanged: (cb: (enabled: boolean) => void) => () => void
  onZoomFactorChanged: (cb: (factor: number) => void) => () => void
  setTitlebar: (theme: TitlebarTheme) => Promise<void>
  runDesktopMenuAction: (action: DesktopMenuAction) => Promise<void>
  setBackgroundColor: (color: string) => Promise<void>
  exportDebugLogs: () => Promise<string>
  recordFatalRendererError: (error: FatalRendererError) => Promise<void>
}

// Declare global API
declare global {
  interface Window {
    api: ElectronAPI
  }
}
