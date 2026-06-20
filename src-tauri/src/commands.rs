//! OpenCode Tauri - Command Handlers
//! This module contains all Tauri command handlers for the OpenCode desktop application.

use std::{
    collections::HashMap,
    env,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, error, info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_shell::ShellExt;

// ============================================================================
// Type Definitions
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerReadyData {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitlebarTheme {
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatalRendererError {
    pub error: String,
    pub url: String,
    pub version: Option<String>,
    pub platform: String,
    pub os: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDirectoryPickerOpts {
    pub multiple: Option<bool>,
    pub title: Option<String>,
    pub default_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFilePickerOpts {
    pub multiple: Option<bool>,
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFilePickerOpts {
    pub title: Option<String>,
    pub default_path: Option<String>,
}

// WSL Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslDistroInfo {
    pub id: String,
    pub name: String,
    pub state: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslServerConfig {
    pub id: String,
    pub distro: String,
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

// Updater Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterState {
    pub status: String,
    pub message: Option<String>,
    pub version: Option<String>,
    pub progress: Option<f64>,
}

impl Default for UpdaterState {
    fn default() -> Self {
        Self {
            status: "idle".to_string(),
            message: None,
            version: None,
            progress: None,
        }
    }
}

// ============================================================================
// App State
// ============================================================================

#[derive(Clone, Default)]
pub struct AppState {
    pub main_window: Arc<Mutex<Option<WebviewWindow>>>,
    pub sidecar_process: Arc<Mutex<Option<tokio::process::Child>>>,
    pub server_url: Arc<Mutex<Option<String>>>,
    pub server_username: Arc<Mutex<Option<String>>>,
    pub server_password: Arc<Mutex<Option<String>>>,
    pub background_color: Arc<Mutex<Option<String>>>,
    pub pinch_zoom_enabled: Arc<Mutex<bool>>,
    pub pending_deep_links: Arc<Mutex<Vec<String>>>,
    pub wsl_servers: Arc<Mutex<HashMap<String, WslServerConfig>>>,
    pub updater_state: Arc<Mutex<UpdaterState>>,
    /// 内存存储，格式: store_name -> { key -> value }
    pub store_data: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}

fn generate_password() -> String {
    let mut rng = rand::thread_rng();
    (0..16)
        .map(|_| {
            let byte = rng.gen::<u8>();
            if byte % 3 == 0 {
                (b'a' + (byte % 26)) as char
            } else if byte % 3 == 1 {
                (b'A' + (byte % 26)) as char
            } else {
                (b'0' + (byte % 10)) as char
            }
        })
        .collect()
}

// ============================================================================
// Sidecar Server Management
// ============================================================================

/// Spawn the OpenCode sidecar server (backend)
pub async fn spawn_sidecar(handle: tauri::AppHandle, window: WebviewWindow) {
    info!("Starting OpenCode sidecar server...");

    // In production, we would:
    // 1. Find the OpenCode backend binary in resources
    // 2. Spawn it as a child process
    // 3. Manage communication
    // 4. Handle lifecycle

    let mut rng = rand::thread_rng();
    let port = rng.gen_range(3000..4000);
    let url = format!("http://127.0.0.1:{}", port);
    let username = "opencode".to_string();
    let password = generate_password();

    // Store in app state
    let state = handle.state::<AppState>();
    *state.server_url.lock().unwrap() = Some(url.clone());
    *state.server_username.lock().unwrap() = Some(username.clone());
    *state.server_password.lock().unwrap() = Some(password);

    info!("Sidecar server: {} (user: {}, pass: ***)", url, username);

    // Show the window after sidecar is ready
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Err(e) = window.show() {
            error!("Failed to show window: {}", e);
        }
    });
}

// ============================================================================
// Window Management Commands
// ============================================================================

#[tauri::command]
pub async fn create_new_window(
    app: tauri::AppHandle,
    title: Option<String>,
    url: Option<String>,
) -> Result<(), String> {
    let label = format!("window-{}", rand::random::<u16>());
    let window = WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::App(url.unwrap_or_else(|| "/".into()).into()),
    )
    .title(title.unwrap_or("OpenCode - New Window".to_string()))
    .inner_size(1280.0, 800.0)
    .build()
    .map_err(|e| e.to_string())?;

    window.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_window_count(app: tauri::AppHandle) -> Result<usize, String> {
    Ok(app.webview_windows().len())
}

#[tauri::command]
pub async fn get_window_focused(window: WebviewWindow) -> Result<bool, String> {
    window.is_focused().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_window_focus(window: WebviewWindow) -> Result<(), String> {
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn show_window(window: WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn hide_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn close_window(window: WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_zoom_factor(window: WebviewWindow) -> Result<f64, String> {
    window.scale_factor().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_zoom_factor(factor: f64) -> Result<(), String> {
    const MIN_ZOOM: f64 = 0.2;
    const MAX_ZOOM: f64 = 10.0;
    let clamped = factor.clamp(MIN_ZOOM, MAX_ZOOM);
    // 在 Tauri 2 中，缩放通过前端 CSS 实现，这里仅记录状态
    info!("Zoom factor set to: {}", clamped);
    Ok(())
}

#[tauri::command]
pub async fn get_pinch_zoom_enabled(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.pinch_zoom_enabled.lock().unwrap())
}

#[tauri::command]
pub async fn set_pinch_zoom_enabled(
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    *state.pinch_zoom_enabled.lock().unwrap() = enabled;
    Ok(())
}

#[tauri::command]
pub async fn set_titlebar_theme(
    theme: TitlebarTheme,
    _window: WebviewWindow,
) -> Result<(), String> {
    // In Tauri, we would customize window decorations
    // For now, just store the preference
    debug!("Titlebar theme set to: {:?}", theme.mode);
    Ok(())
}

#[tauri::command]
pub async fn set_background_color(
    color: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    *state.background_color.lock().unwrap() = Some(color);
    Ok(())
}

#[tauri::command]
pub async fn set_window_title(title: String, window: WebviewWindow) -> Result<(), String> {
    let _ = window.set_title(&title);
    Ok(())
}

// ============================================================================
// Sidecar & Server Commands
// ============================================================================

#[tauri::command]
pub async fn kill_sidecar(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // 在 await 前释放 MutexGuard（避免跨 await 持有非 Send 类型）
    let child = state.sidecar_process.lock().unwrap().take();
    if let Some(mut child) = child {
        child.kill().await.map_err(|e| e.to_string())?;
    }
    *state.server_url.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub async fn await_initialization(
    state: tauri::State<'_, AppState>,
) -> Result<ServerReadyData, String> {
    let server_url = state.server_url.lock().unwrap().clone();
    let server_username = state.server_username.lock().unwrap().clone();
    let server_password = state.server_password.lock().unwrap().clone();

    Ok(ServerReadyData {
        url: server_url.unwrap_or_default(),
        username: server_username,
        password: server_password,
    })
}

#[tauri::command]
pub async fn consume_initial_deep_links(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let mut links = state.pending_deep_links.lock().unwrap();
    Ok(links.drain(..).collect())
}

#[tauri::command]
pub async fn get_default_server_url(
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    Ok(state.server_url.lock().unwrap().clone())
}

#[tauri::command]
pub async fn set_default_server_url(
    url: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    *state.server_url.lock().unwrap() = url;
    Ok(())
}

// ============================================================================
// App Checking Commands
// ============================================================================

#[tauri::command]
pub async fn check_app_exists(app_name: String) -> Result<bool, String> {
    check_app_exists_impl(&app_name).await
}

async fn check_app_exists_impl(app_name: &str) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("mdfind")
            .arg(format!("kMDItemDisplayName == '{}'", app_name))
            .output()
            .map_err(|e| e.to_string())?;
        Ok(!output.stdout.is_empty())
    }
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("where")
            .arg(app_name)
            .output()
            .map_err(|e| e.to_string())?;
        Ok(output.status.success())
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("which")
            .arg(app_name)
            .output()
            .map_err(|e| e.to_string())?;
        Ok(output.status.success())
    }
}

#[tauri::command]
pub async fn resolve_app_path(app_name: String) -> Result<Option<String>, String> {
    resolve_app_path_impl(&app_name).await
}

async fn resolve_app_path_impl(app_name: &str) -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("mdfind")
            .arg(format!("kMDItemDisplayName == '{}'", app_name))
            .output()
            .map_err(|e| e.to_string())?;
        if output.stdout.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                String::from_utf8(output.stdout)
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string(),
            ))
        }
    }
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("where")
            .arg(app_name)
            .output()
            .map_err(|e| e.to_string())?;
        if output.stdout.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                String::from_utf8(output.stdout)
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string(),
            ))
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("which")
            .arg(app_name)
            .output()
            .map_err(|e| e.to_string())?;
        if output.stdout.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                String::from_utf8(output.stdout)
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string(),
            ))
        }
    }
}

// ============================================================================
// File Picker Commands
// ============================================================================

#[tauri::command]
pub async fn open_directory_picker(
    window: WebviewWindow,
    opts: Option<OpenDirectoryPickerOpts>,
) -> Result<Option<Vec<String>>, String> {
    let mut builder = window.dialog().file();
    if let Some(ref opts) = opts {
        if let Some(ref title) = opts.title {
            builder = builder.set_title(title.clone());
        }
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<FilePath>>();
    builder.pick_folder(move |result| {
        let _ = tx.send(result);
    });
    let path = rx.await.map_err(|_| "对话框被取消".to_string())?;
    Ok(path.map(|p| vec![p.to_string()]))
}

#[tauri::command]
pub async fn open_file_picker(
    window: WebviewWindow,
    opts: Option<OpenFilePickerOpts>,
) -> Result<Option<serde_json::Value>, String> {
    let multiple = opts.as_ref().and_then(|o| o.multiple).unwrap_or(false);
    let extensions = opts.as_ref().and_then(|o| o.extensions.clone());

    if multiple {
        let mut builder = window.dialog().file().add_filter("All Files", &["*"]);
        if let Some(title) = opts.as_ref().and_then(|o| o.title.as_ref()) {
            builder = builder.set_title(title.clone());
        }
        if let Some(ref exts) = extensions {
            let filter_exts: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
            builder = builder.add_filter("Files", &filter_exts);
        }

        let (tx, rx) = tokio::sync::oneshot::channel::<Option<Vec<FilePath>>>();
        builder.pick_files(move |result| {
            let _ = tx.send(result);
        });
        let paths = rx.await.map_err(|_| "对话框被取消".to_string())?;
        Ok(paths.map(|paths| {
            let token = generate_token();
            let files: Vec<_> = paths
                .iter()
                .map(|p| {
                    let path_str = p.to_string();
                    let metadata = std::fs::metadata(&path_str).ok();
                    let size = metadata.map(|m| m.len()).unwrap_or(0);
                    let name = Path::new(&path_str)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    serde_json::json!({
                        "path": path_str,
                        "name": name,
                        "size": size
                    })
                })
                .collect();
            serde_json::json!({ "token": token, "files": files })
        }))
    } else {
        let mut builder = window.dialog().file();
        if let Some(title) = opts.as_ref().and_then(|o| o.title.as_ref()) {
            builder = builder.set_title(title.clone());
        }
        if let Some(ref exts) = extensions {
            let filter_exts: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
            builder = builder.add_filter("Files", &filter_exts);
        }

        let (tx, rx) = tokio::sync::oneshot::channel::<Option<FilePath>>();
        builder.pick_file(move |result| {
            let _ = tx.send(result);
        });
        let path = rx.await.map_err(|_| "对话框被取消".to_string())?;
        Ok(path.map(|path| {
            let token = generate_token();
            let path_str = path.to_string();
            let metadata = std::fs::metadata(&path_str).ok();
            let size = metadata.map(|m| m.len()).unwrap_or(0);
            let name = Path::new(&path_str)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            serde_json::json!({
                "token": token,
                "files": [{
                    "path": path_str,
                    "name": name,
                    "size": size
                }]
            })
        }))
    }
}

#[tauri::command]
pub async fn save_file_picker(
    window: WebviewWindow,
    opts: Option<SaveFilePickerOpts>,
) -> Result<Option<String>, String> {
    let mut builder = window.dialog().file();
    if let Some(ref opts) = opts {
        if let Some(ref title) = opts.title {
            builder = builder.set_title(title.clone());
        }
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<FilePath>>();
    builder.save_file(move |result| {
        let _ = tx.send(result);
    });
    let path = rx.await.map_err(|_| "对话框被取消".to_string())?;
    Ok(path.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn read_picked_file(_token: String, path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn release_picked_files(_token: String) -> Result<(), String> {
    // In Tauri, file handles are managed differently
    // This is a no-op for now
    Ok(())
}

// ============================================================================
// System Commands
// ============================================================================

#[tauri::command]
#[allow(deprecated)]
pub async fn open_link(window: WebviewWindow, url: String) -> Result<(), String> {
    window.shell().open(&url, None).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[allow(deprecated)]
pub async fn open_path(
    window: WebviewWindow,
    path: String,
    app: Option<String>,
) -> Result<(), String> {
    if let Some(app_name) = app {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .args(["-a", &app_name, &path])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new(app_name)
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new(app_name)
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    } else {
        window
            .shell()
            .open(&path, None)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn read_clipboard_image(
    window: WebviewWindow,
) -> Result<Option<serde_json::Value>, String> {
    let image = window.clipboard().read_image().map_err(|e| e.to_string())?;

    let buffer: Vec<u8> = image.rgba().to_vec();
    Ok(Some(serde_json::json!({
        "buffer": buffer,
        "width": image.width(),
        "height": image.height()
    })))
}

#[tauri::command]
pub async fn show_notification(
    window: WebviewWindow,
    title: String,
    body: Option<String>,
) -> Result<(), String> {
    window
        .notification()
        .builder()
        .title(&title)
        .body(body.as_deref().unwrap_or(""))
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn relaunch() -> Result<(), String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    std::process::Command::new(exe)
        .spawn()
        .map_err(|e| e.to_string())?;
    std::process::exit(0);
}

// ============================================================================
// Display Backend Commands
// ============================================================================

#[tauri::command]
pub async fn get_display_backend() -> Result<Option<String>, String> {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return Ok(Some("wayland".to_string()));
        } else if std::env::var("DISPLAY").is_ok() {
            return Ok(Some("x11".to_string()));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn set_display_backend(backend: Option<String>) -> Result<(), String> {
    warn!("Changing display backend requires restart: {:?}", backend);
    Ok(())
}

// ============================================================================
// Store Commands
// ============================================================================

#[tauri::command]
pub async fn store_get(
    name: String,
    key: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    let store = state.store_data.lock().unwrap();
    Ok(store.get(&name).and_then(|m| m.get(&key)).cloned())
}

#[tauri::command]
pub async fn store_set(
    name: String,
    key: String,
    value: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.store_data.lock().unwrap();
    store.entry(name).or_default().insert(key, value);
    Ok(())
}

#[tauri::command]
pub async fn store_delete(
    name: String,
    key: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.store_data.lock().unwrap();
    if let Some(map) = store.get_mut(&name) {
        map.remove(&key);
    }
    Ok(())
}

#[tauri::command]
pub async fn store_clear(name: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut store = state.store_data.lock().unwrap();
    store.remove(&name);
    Ok(())
}

#[tauri::command]
pub async fn store_keys(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let store = state.store_data.lock().unwrap();
    Ok(store
        .get(&name)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default())
}

#[tauri::command]
pub async fn store_length(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    let store = state.store_data.lock().unwrap();
    Ok(store.get(&name).map(|m| m.len()).unwrap_or(0))
}

// ============================================================================
// Logging & Debug Commands
// ============================================================================

#[tauri::command]
pub async fn export_debug_logs() -> Result<String, String> {
    // In a real implementation, collect logs from various sources
    Ok("Debug logs exported".to_string())
}

#[tauri::command]
pub async fn record_fatal_renderer_error(error: FatalRendererError) -> Result<(), String> {
    error!("Fatal renderer error: {:?}", error);
    // In production, save to disk or send to analytics
    Ok(())
}

// ============================================================================
// Updater Commands
// ============================================================================

#[tauri::command]
pub async fn updater_subscribe(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let updater_state = state.updater_state.lock().unwrap();
    // Send current state to renderer
    window
        .emit("updater-state", &*updater_state)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn updater_unsubscribe(_window: WebviewWindow) -> Result<(), String> {
    // Remove listener
    Ok(())
}

#[tauri::command]
pub async fn updater_check(state: tauri::State<'_, AppState>) -> Result<UpdaterState, String> {
    // 在 await 前释放 MutexGuard
    {
        let mut updater_state = state.updater_state.lock().unwrap();
        updater_state.status = "checking".to_string();
        updater_state.message = Some("Checking for updates...".to_string());
    }

    // 模拟检查（此时 MutexGuard 已释放）
    tokio::time::sleep(Duration::from_secs(1)).await;

    let result = {
        let mut updater_state = state.updater_state.lock().unwrap();
        updater_state.status = "up-to-date".to_string();
        updater_state.message = Some("No updates available".to_string());
        updater_state.clone()
    };

    Ok(result)
}

#[tauri::command]
pub async fn updater_install(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // 在 await 前释放 MutexGuard
    {
        let updater_state = state.updater_state.lock().unwrap();
        if updater_state.status != "ready" {
            return Err("Update is not ready to install".to_string());
        }
    }

    {
        let mut updater_state = state.updater_state.lock().unwrap();
        updater_state.status = "installing".to_string();
        updater_state.message = Some("Installing update...".to_string());
    }

    // 模拟安装（此时 MutexGuard 已释放）
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Restart the application
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    std::process::Command::new(exe)
        .spawn()
        .map_err(|e| e.to_string())?;
    std::process::exit(0);
}

// ============================================================================
// WSL Commands (Windows only)
// ============================================================================

#[tauri::command]
pub async fn wsl_servers_get_state(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let wsl_servers = state.wsl_servers.lock().unwrap();
    Ok(serde_json::json!(wsl_servers.clone()))
}

#[tauri::command]
pub async fn wsl_servers_probe_runtime() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        // Check if WSL is installed
        let output = std::process::Command::new("wsl")
            .arg("--list")
            .arg("--verbose")
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
            return Ok(serde_json::json!({
                "installed": true,
                "version": "2",
                "distros": stdout
            }));
        }

        return Ok(serde_json::json!({
            "installed": false,
            "error": String::from_utf8(output.stderr).unwrap_or_default()
        }));
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(serde_json::json!({"installed": false, "error": "Not available on this platform"}))
    }
}

#[tauri::command]
pub async fn wsl_servers_refresh_distros() -> Result<Vec<WslDistroInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("wsl")
            .arg("--list")
            .arg("--verbose")
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "Failed to list WSL distros".to_string()));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;

        // Parse WSL output
        let mut distros = Vec::new();
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                distros.push(WslDistroInfo {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    state: parts[2].to_string(),
                    version: parts.get(3).cloned().map(|s| s.to_string()),
                });
            }
        }

        Ok(distros)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("WSL is only available on Windows".to_string())
    }
}

#[tauri::command]
#[allow(deprecated)]
pub async fn wsl_servers_install_wsl(_window: WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Open Microsoft Store to install WSL
        _window
            .shell()
            .open("ms-windows-store://pdp/?ProductId=9P9TQF7MRM4R", None)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("WSL is only available on Windows".to_string())
    }
}

#[tauri::command]
pub async fn wsl_servers_probe_distro(_name: String) -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        // Check if specific distro exists
        let output = std::process::Command::new("wsl")
            .arg("-d")
            .arg(&_name)
            .arg("--exec")
            .arg("echo")
            .arg("probed")
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            return Ok(serde_json::json!({
                "available": true,
                "distro": _name,
                "message": "Distro is accessible"
            }));
        }

        return Ok(serde_json::json!({
            "available": false,
            "distro": _name,
            "error": String::from_utf8(output.stderr).unwrap_or_default()
        }));
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("WSL is only available on Windows".to_string())
    }
}

#[tauri::command]
pub async fn wsl_servers_install_distro(_name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        warn!("Installing WSL distro: {}", _name);
        // In production, this would use wsl --install -d <name>
        // For now, just log
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("WSL is only available on Windows".to_string())
    }
}

#[tauri::command]
pub async fn wsl_servers_open_terminal(_name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("wsl")
            .arg("-d")
            .arg(&_name)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("WSL is only available on Windows".to_string())
    }
}

#[tauri::command]
pub async fn wsl_servers_add_server(
    config: WslServerConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut wsl_servers = state.wsl_servers.lock().unwrap();
    wsl_servers.insert(config.id.clone(), config);
    Ok(())
}

#[tauri::command]
pub async fn wsl_servers_remove_server(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut wsl_servers = state.wsl_servers.lock().unwrap();
    wsl_servers.remove(&id);
    Ok(())
}

#[tauri::command]
pub async fn wsl_servers_start_server(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let wsl_servers = state.wsl_servers.lock().unwrap();
    if let Some(server) = wsl_servers.get(&id) {
        warn!(
            "Starting WSL server: {} on port {}",
            server.name, server.port
        );
        // In production, start the server process
        Ok(())
    } else {
        Err(format!("WSL server not found: {}", id))
    }
}

// ============================================================================
// Menu Commands
// ============================================================================

#[tauri::command]
pub async fn create_desktop_menu(_window: WebviewWindow) -> Result<(), String> {
    // In Tauri, menus are created differently
    // For now, just log
    debug!("Creating desktop menu...");
    Ok(())
}

#[tauri::command]
pub async fn run_desktop_menu_action(action: String, window: WebviewWindow) -> Result<(), String> {
    debug!("Running desktop menu action: {}", action);

    // Handle common menu actions
    match action.as_str() {
        "check-for-updates" => {
            // Trigger update check
            window.emit("menu-action", "check-for-updates").ok();
        }
        "relaunch" => {
            if let Ok(exe) = env::current_exe() {
                let _ = std::process::Command::new(exe).spawn();
                std::process::exit(0);
            }
        }
        "quit" => {
            // Close all windows and quit
            window.close().ok();
        }
        _ => {
            warn!("Unknown menu action: {}", action);
        }
    }

    Ok(())
}

// ============================================================================
// Deep Link Commands
// ============================================================================

#[tauri::command]
pub async fn register_deep_link_handler(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // In Tauri, deep links can be handled via custom protocol
    debug!("Registering deep link handler...");

    // Store the window reference for later use
    *state.main_window.lock().unwrap() = Some(window);

    Ok(())
}

// ============================================================================
// Utility Commands
// ============================================================================

#[tauri::command]
pub async fn parse_markdown(markdown: String) -> Result<String, String> {
    // Simple markdown parsing
    // In production, use a proper markdown library
    Ok(markdown)
}

#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok("1.17.8".to_string())
}

#[tauri::command]
pub async fn get_platform() -> Result<String, String> {
    Ok(std::env::consts::OS.to_string())
}

#[tauri::command]
pub async fn get_arch() -> Result<String, String> {
    Ok(std::env::consts::ARCH.to_string())
}
