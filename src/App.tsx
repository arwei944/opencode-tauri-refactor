import { createSignal, onMount } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/api/shell'

export default function App() {
  const [greetMsg, setGreetMsg] = createSignal('')
  const [name, setName] = createSignal('')
  const [windowCount, setWindowCount] = createSignal(0)
  const [zoomFactor, setZoomFactor] = createSignal(1.0)
  const [pinchZoomEnabled, setPinchZoomEnabled] = createSignal(false)
  const [serverUrl, setServerUrl] = createSignal('')
  const [backgroundColor, setBackgroundColor] = createSignal('')
  const [storeValue, setStoreValue] = createSignal('')
  const [storeKey, setStoreKey] = createSignal('test-key')

  // Initialize
  onMount(async () => {
    try {
      // Get server URL
      const url = await invoke('get_default_server_url') as string | null
      if (url) setServerUrl(url)

      // Get window count
      const count = await invoke('get_window_count') as number
      setWindowCount(count)

      // Get zoom factor
      const zoom = await invoke('get_zoom_factor') as number
      setZoomFactor(zoom)

      // Get pinch zoom setting
      const pinch = await invoke('get_pinch_zoom_enabled') as boolean
      setPinchZoomEnabled(pinch)

      // Test store
      const stored = await invoke('store_get', { name: 'test-store', key: 'test-key' }) as string | null
      if (stored) setStoreValue(stored)
    } catch (error) {
      console.error('Initialization error:', error)
    }
  })

  // Command handlers
  const greet = async () => {
    try {
      const msg = await invoke('greet', { name: name() }) as string
      setGreetMsg(msg)
    } catch (error) {
      console.error('Greet error:', error)
    }
  }

  const awaitInit = async () => {
    try {
      const data = await invoke('await_initialization') as { url: string; username: string; password: string }
      setServerUrl(data.url)
      console.log('Initialization:', data)
    } catch (error) {
      console.error('Init error:', error)
    }
  }

  const checkApp = async (appName: string) => {
    try {
      const exists = await invoke('check_app_exists', { appName }) as boolean
      alert(`${appName} exists: ${exists}`)
    } catch (error) {
      console.error('Check app error:', error)
    }
  }

  const openDirectory = async () => {
    try {
      const result = await invoke('open_directory_picker', { opts: { title: 'Select Folder' } }) as string[] | null
      if (result) {
        console.log('Selected directory:', result)
        alert(`Selected: ${result.join(', ')}`)
      }
    } catch (error) {
      console.error('Directory picker error:', error)
    }
  }

  const openFile = async () => {
    try {
      const result = await invoke('open_file_picker', { 
        opts: { 
          title: 'Select File',
          extensions: ['txt', 'md', 'json'] 
        } 
      }) as { token: string; files: any[] } | null
      if (result) {
        console.log('Selected file:', result)
        alert(`Selected ${result.files.length} file(s)`)
      }
    } catch (error) {
      console.error('File picker error:', error)
    }
  }

  const saveFile = async () => {
    try {
      const result = await invoke('save_file_picker', { opts: { title: 'Save File' } }) as string | null
      if (result) {
        console.log('Save path:', result)
        alert(`Save to: ${result}`)
      }
    } catch (error) {
      console.error('Save picker error:', error)
    }
  }

  const showNotification = async () => {
    try {
      await invoke('show_notification', { 
        title: 'OpenCode', 
        body: 'Hello from Tauri!' 
      })
    } catch (error) {
      console.error('Notification error:', error)
    }
  }

  const setZoom = async (factor: number) => {
    try {
      await invoke('set_zoom_factor', { factor })
      setZoomFactor(factor)
    } catch (error) {
      console.error('Zoom error:', error)
    }
  }

  const togglePinchZoom = async () => {
    try {
      const enabled = !pinchZoomEnabled()
      await invoke('set_pinch_zoom_enabled', { enabled })
      setPinchZoomEnabled(enabled)
    } catch (error) {
      console.error('Pinch zoom error:', error)
    }
  }

  const saveStoreValue = async () => {
    try {
      await invoke('store_set', { 
        name: 'test-store', 
        key: storeKey(), 
        value: storeValue() 
      })
      alert('Value stored!')
    } catch (error) {
      console.error('Store error:', error)
    }
  }

  const getStoreValue = async () => {
    try {
      const value = await invoke('store_get', { 
        name: 'test-store', 
        key: storeKey() 
      }) as string | null
      if (value) {
        setStoreValue(value)
        alert(`Got: ${value}`)
      } else {
        alert('No value found')
      }
    } catch (error) {
      console.error('Get store error:', error)
    }
  }

  const openExternalLink = async () => {
    try {
      await open('https://opencode.ai')
    } catch (error) {
      console.error('Open link error:', error)
    }
  }

  const relaunchApp = async () => {
    try {
      await invoke('relaunch')
    } catch (error) {
      console.error('Relaunch error:', error)
    }
  }

  return (
    <main class="container" style={{ 
      'min-height': '100vh', 
      'padding': '2rem',
      'max-width': '1200px',
      'margin': '0 auto'
    }}>
      <h1>OpenCode Tauri Desktop</h1>
      <p>Welcome to the Tauri version of OpenCode!</p>

      <section style={{ 'margin-bottom': '2rem', 'padding': '1rem', 'border': '1px solid #333', 'border-radius': '8px' }}>
        <h2>Server Status</h2>
        <p>Server URL: {serverUrl() || 'Not initialized'}</p>
        <button onClick={awaitInit}>Await Initialization</button>
      </section>

      <section style={{ 'margin-bottom': '2rem', 'padding': '1rem', 'border': '1px solid #333', 'border-radius': '8px' }}>
        <h2>Basic Commands</h2>
        <div style={{ 'display': 'flex', 'gap': '1rem', 'margin-bottom': '1rem' }}>
          <input
            id="greet-input"
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Enter a name..."
            value={name()}
            style={{ 'padding': '0.5rem', 'flex': '1' }}
          />
          <button type="button" onClick={greet} style={{ 'padding': '0.5rem 1rem' }}>
            Greet
          </button>
        </div>
        <p>{greetMsg()}</p>
      </section>

      <section style={{ 'margin-bottom': '2rem', 'padding': '1rem', 'border': '1px solid #333', 'border-radius': '8px' }}>
        <h2>Window Management</h2>
        <p>Window Count: {windowCount()}</p>
        <p>Zoom Factor: {zoomFactor().toFixed(2)}</p>
        <div style={{ 'display': 'flex', 'gap': '1rem', 'flex-wrap': 'wrap' }}>
          <button onClick={() => setZoom(0.8)}>Zoom Out</button>
          <button onClick={() => setZoom(1.0)}>Reset Zoom</button>
          <button onClick={() => setZoom(1.5)}>Zoom In</button>
        </div>
        <p>Pinch Zoom Enabled: {pinchZoomEnabled() ? 'Yes' : 'No'}</p>
        <button onClick={togglePinchZoom}>Toggle Pinch Zoom</button>
      </section>

      <section style={{ 'margin-bottom': '2rem', 'padding': '1rem', 'border': '1px solid #333', 'border-radius': '8px' }}>
        <h2>File Operations</h2>
        <div style={{ 'display': 'flex', 'gap': '1rem', 'flex-wrap': 'wrap' }}>
          <button onClick={openDirectory}>Open Directory</button>
          <button onClick={openFile}>Open File</button>
          <button onClick={saveFile}>Save File</button>
        </div>
      </section>

      <section style={{ 'margin-bottom': '2rem', 'padding': '1rem', 'border': '1px solid #333', 'border-radius': '8px' }}>
        <h2>Storage</h2>
        <div style={{ 'display': 'flex', 'gap': '1rem', 'margin-bottom': '1rem' }}>
          <input
            onChange={(e) => setStoreKey(e.currentTarget.value)}
            placeholder="Key"
            value={storeKey()}
            style={{ 'padding': '0.5rem' }}
          />
          <input
            onChange={(e) => setStoreValue(e.currentTarget.value)}
            placeholder="Value"
            value={storeValue()}
            style={{ 'padding': '0.5rem', 'flex': '1' }}
          />
        </div>
        <div style={{ 'display': 'flex', 'gap': '1rem' }}>
          <button onClick={saveStoreValue}>Store Value</button>
          <button onClick={getStoreValue}>Get Value</button>
        </div>
      </section>

      <section style={{ 'margin-bottom': '2rem', 'padding': '1rem', 'border': '1px solid #333', 'border-radius': '8px' }}>
        <h2>System</h2>
        <div style={{ 'display': 'flex', 'gap': '1rem', 'flex-wrap': 'wrap' }}>
          <button onClick={() => checkApp('vscode')}'>Check VSCode</button>
          <button onClick={() => checkApp('chrome')}'>Check Chrome</button>
          <button onClick={showNotification}>Show Notification</button>
          <button onClick={openExternalLink}>Open OpenCode.ai</button>
          <button onClick={relaunchApp}>Relaunch App</button>
        </div>
      </section>

      <section style={{ 'margin-bottom': '2rem', 'padding': '1rem', 'border': '1px solid #333', 'border-radius': '8px' }}>
        <h2>Configuration</h2>
        <p>Background Color: {backgroundColor() || 'Default'}</p>
      </section>
    </main>
  )
}
