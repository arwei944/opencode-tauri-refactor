import { createSignal, onMount } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import './index.css'

export default function App() {
  const [windowCount, setWindowCount] = createSignal(0)
  const [serverUrl, setServerUrl] = createSignal('')
  const [appVersion, setAppVersion] = createSignal('')

  onMount(async () => {
    try {
      const count = await invoke('get_window_count') as number
      setWindowCount(count)
      
      const url = await invoke('get_default_server_url') as string | null
      setServerUrl(url || '')
      
      const version = await invoke('get_app_version') as string
      setAppVersion(version)
    } catch (error) {
      console.error('Init error:', error)
    }
  })

  const openExternalLink = async () => {
    try {
      await open('https://opencode.ai')
    } catch (error) {
      console.error('Open link error:', error)
    }
  }

  const showNotification = async () => {
    try {
      await invoke('show_notification', { 
        title: 'OpenCode', 
        body: 'Welcome to Tauri version!' 
      })
    } catch (error) {
      console.error('Notification error:', error)
    }
  }

  return (
    <div class="container">
      <header>
        <h1>OpenCode Desktop</h1>
        <p>Tauri Version - v{appVersion()}</p>
      </header>

      <main>
        <section>
          <h2>Status</h2>
          <p>Window Count: {windowCount()}</p>
          <p>Server URL: {serverUrl() || 'Not set'}</p>
        </section>

        <section>
          <h2>Actions</h2>
          <div style={{ display: 'flex', gap: '1rem', 'flex-wrap': 'wrap' }}>
            <button onClick={openExternalLink}>Open OpenCode.ai</button>
            <button onClick={showNotification}>Show Notification</button>
          </div>
        </section>
      </main>

      <footer>
        <p>Built with Tauri + SolidJS</p>
      </footer>
    </div>
  )
}
