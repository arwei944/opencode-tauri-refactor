//! OpenCode Tauri - Command Handlers
//! This module contains all Tauri command handlers for the OpenCode desktop application.

use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, error, info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::{
    api::path::BaseDirectory,
    async_runtime::spawn,
    Manager, Runtime, Window, WindowEvent, WindowUrl,
};

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

// ============================================================================
// App State
// ============================================================================

#[derive(Default)]
pub struct AppState {
    pub main_window: Option<Window>,
    pub sidecar_process: Option<tokio::process::Child>,
    pub server_url: Arc<Mutex<Option<String>>>,
    pub server_username: Arc<Mutex<Option<String>>>,
    pub server_password: Arc<Mutex<Option<String>>>,
    pub background_color: Arc<Mutex<Option<String>>>,
    pub pinch_zoom_enabled: Arc<Mutex<bool>>,
    pub pending_deep_links: Arc<Mutex<Vec<String>>>,
    pub wsl_servers: Arc<Mutex<HashMap<String, WslServerConfig>>>,
    pub updater_state: Arc<Mutex<UpdaterState>>,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
}

fn generate_password() -> String {
    let mut rng = rand::thread_rng();
    (0..16).map(|_| {
        let byte = rng.gen::<u8>();
        if byte % 3 == 0 {
            (b'a' + (byte % 26)) as char
        } else if byte % 3 == 1 {
            (b'A' + (byte % 26)) as char
        } else {
            (b'0' + (byte % 10)) as char
        }
    }).collect()
}

// ============================================================================
// Sidecar Server Management
// ============================================================================

/// Spawn the OpenCode sidecar server (backend)
pub async fn spawn_sidecar(handle: tauri::AppHandle, window: Window) {
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
    if let Some(state) = handle.state::<AppState>() {
        let mut state = state.lock().unwrap();
        *state.server_url.lock().unwrap() = Some(url.clone());
        *state.server_username.lock().unwrap() = Some(username.clone());
        *state.server_password.lock().unwrap() = Some(password);
    }

    info!("Sidecar server: {} (user: {}, pass: ***)", url, username);

    // Show the window after sidecar is ready
    spawn(async move {
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
    let window = WindowBuilder::new(&app, WindowUrl::App(url.unwrap_or("/".into())))
        .title(title.unwrap_or("OpenCode - New Window".to_string()))
        .inner_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 1280.0,
            height: 800.0,
        }))
        .build()
        .map_err(|e| e.to_string())?;

    window.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_window_count(manager: tauri::Manager<Runtime>) -> Result<usize, String> {
    Ok(manager.webviews().len())
}

#[tauri::command]
pub async fn get_window_focused(window: Window) -> Result<bool, String> {
    Ok(window.is_focused().await)
}

#[tauri::command]
pub async fn set_window_focus(window: Window) -> Result<(), String> {
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn show_window(window: Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn hide_window(window: Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn close_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_zoom_factor(window: Window) -> Result<f64, String> {
    Ok(window.scale_factor().await)
}

#[tauri::command]
pub async fn set_zoom_factor(factor: f64, window: Window) -> Result<(), String> {
    const MIN_ZOOM: f64 = 0.2;
    const MAX_ZOOM: f64 = 10.0;
    window.set_scale_factor(factor.clamp(MIN_ZOOM, MAX_ZOOM));
    Ok(())
}

#[tauri::command]
pub async fn get_pinch_zoom_enabled(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.pinch_zoom_enabled.lock().unwrap())
}

#[tauri::command]
pub async fn set_pinch_zoom_enabled(enabled: bool, state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.pinch_zoom_enabled.lock().unwrap() = enabled;
    Ok(())
}

#[tauri::command]
pub async fn set_titlebar_theme(theme: TitlebarTheme, window: Window) -> Result<(), String> {
    // In Tauri, we would customize window decorations
    // For now, just store the preference
    debug!("Titlebar theme set to: {:?}", theme.mode);
    Ok(())
}

#[tauri::command]
pub async fn set_background_color(color: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.background_color.lock().unwrap() = Some(color);
    Ok(())
}

#[tauri::command]
pub async fn set_window_title(title: String, window: Window) -> Result<(), String> {
    window.set_title(&title).map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// Sidecar & Server Commands
// ============================================================================

#[tauri::command]
pub async fn kill_sidecar(state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(mut child) = state.sidecar_process.lock().unwrap().take() {
        child.kill().await.map_err(|e| e.to_string())?;
    }
    *state.server_url.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub async fn await_initialization(state: tauri::State<'_, AppState>) -> Result<ServerReadyData, String> {
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
pub async fn consume_initial_deep_links(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut links = state.pending_deep_links.lock().unwrap();
    Ok(links.drain(..).collect())
}

#[tauri::command]
pub async fn get_default_server_url(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state.server_url.lock().unwrap().clone())
}

#[tauri::command]
pub async fn set_default_server_url(url: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
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
    window: Window,
    opts: Option<OpenDirectoryPickerOpts>,
) -> Result<Option<Vec<String>>, String> {
    let title = opts
        .clone()
        .and_then(|o| o.title)
        .unwrap_or_else(|| "Choose a folder".to_string());
    let default_path = opts.and_then(|o| o.default_path);

    let path = window
        .dialog()
        .pick_folder(title, default_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(path.map(|p| vec![p]))
}

#[tauri::command]
pub async fn open_file_picker(
    window: Window,
    opts: Option<OpenFilePickerOpts>,
) -> Result<Option<serde_json::Value>, String> {
    let title = opts
        .clone()
        .and_then(|o| o.title)
        .unwrap_or_else(|| "Choose a file".to_string());
    let default_path = opts.clone().and_then(|o| o.default_path);
    let extensions = opts.and_then(|o| o.extensions);

    let mut builder = window.dialog().add_filter("All Files", &["*"]);
    if let Some(exts) = extensions {
        builder = builder.add_filter("Files", &exts);
    }

    let result = if opts.map(|o| o.multiple).unwrap_or(false) {
        window
            .dialog()
            .pick_files(title, default_path, builder)
            .await
            .map_err(|e| e.to_string())?
    } else {
        window
            .dialog()
            .pick_file(title, default_path, builder)
            .await
            .map_err(|e| e.to_string())?
            .map(|p| vec![p])
    };

    Ok(result.map(|paths| {
        let token = generate_token();
        let files: Vec<_> = paths
            .iter()
            .map(|p| {
                let metadata = std::fs::metadata(p).ok();
                let size = metadata.map(|m| m.len()).unwrap_or(0);
                let name = Path::new(p)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                serde_json::json!({
                    "path": p,
                    "name": name,
                    "size": size
                })
            })
            .collect();
        serde_json::json!({ "token": token, "files": files })
    }))
}

#[tauri::command]
pub async fn save_file_picker(
    window: Window,
    opts: Option<SaveFilePickerOpts>,
) -> Result<Option<String>, String> {
    let title = opts
        .clone()
        .and_then(|o| o.title)
        .unwrap_or_else(|| "Save file".to_string());
    let default_path = opts.and_then(|o| o.default_path);

    window
        .dialog()
        .save_file(title, default_path)
        .await
        .map_err(|e| e.to_string())
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
pub async fn open_link(window: Window, url: String) -> Result<(), String> {
    window
        .shell()
        .open(url, None)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_path(path: String, app: Option<String>) -> Result<(), String> {
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
        tauri::api::shell::open(&Path::new(&path), None)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn read_clipboard_image(window: Window) -> Result<Option<serde_json::Value>, String> {
    let image = window
        .clipboard()
        .read_image()
        .await
        .map_err(|e| e.to_string())?;

    Ok(image.map(|img| {
        serde_json::json!({
            "buffer": img.bytes,
            "width": img.width,
            "height": img.height
        })
    }))
}

#[tauri::command]
pub async fn show_notification(window: Window, title: String, body: Option<String>) -> Result<(), String> {
    window
        .notification()
        .show(&title, body.as_deref(), None)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn relaunch() -> Result<(), String> {
    tauri::api::process::restart(&env::current_exe().unwrap());
    Ok(())
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
    store: tauri::State<'_, tauri_plugin_store::Store<Mutex<HashMap<String, serde_json::Value>>>>,
) -> Result<Option<String>, String> {
    let state = store.lock().unwrap();
    if let Some(serde_json::Value::Object(map)) = state.get(&name) {
        if let Some(value) = map.get(&key) {
            return Ok(Some(value.to_string()));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn store_set(
    name: String,
    key: String,
    value: String,
    store: tauri::State<'_, tauri_plugin_store::Store<Mutex<HashMap<String, serde_json::Value>>>>,
) -> Result<(), String> {
    let mut state = store.lock().unwrap();
    let store_entry = state.entry(name.clone()).or_insert_with(|| serde_json::json!({}));
    if let Some(obj) = store_entry.as_object_mut() {
        obj.insert(key.clone(), serde_json::json!(value));
    }
    Ok(())
}

#[tauri::command]
pub async fn store_delete(
    name: String,
    key: String,
    store: tauri::State<'_, tauri_plugin_store::Store<Mutex<HashMap<String, serde_json::Value>>>>,
) -> Result<(), String> {
    let mut state = store.lock().unwrap();
    if let Some(serde_json::Value::Object(map)) = state.get_mut(&name) {
        map.remove(&key);
    }
    Ok(())
}

#[tauri::command]
pub async fn store_clear(
    name: String,
    store: tauri::State<'_, tauri_plugin_store::Store<Mutex<HashMap<String, serde_json::Value>>>>,
) -> Result<(), String> {
    let mut state = store.lock().unwrap();
    state.remove(&name);
    Ok(())
}

#[tauri::command]
pub async fn store_keys(
    name: String,
    store: tauri::State<'_, tauri_plugin_store::Store<Mutex<HashMap<String, serde_json::Value>>>>,
) -> Result<Vec<String>, String> {
    let state = store.lock().unwrap();
    if let Some(serde_json::Value::Object(map)) = state.get(&name) {
        return Ok(map.keys().cloned().collect());
    }
    Ok(vec![])
}

#[tauri::command]
pub async fn store_length(
    name: String,
    store: tauri::State<'_, tauri_plugin_store::Store<Mutex<HashMap<String, serde_json::Value>>>>,
) -> Result<usize, String> {
    let state = store.lock().unwrap();
    if let Some(serde_json::Value::Object(map)) = state.get(&name) {
        return Ok(map.len());
    }
    Ok(0)
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
    window: Window,
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
pub async fn updater_unsubscribe(_window: Window) -> Result<(), String> {
    // Remove listener
    Ok(())
}

#[tauri::command]
pub async fn updater_check(state: tauri::State<'_, AppState>) -> Result<UpdaterState, String> {
    let mut updater_state = state.updater_state.lock().unwrap();
    
    // Mock implementation - in production, check for updates
    updater_state.status = "checking".to_string();
    updater_state.message = Some("Checking for updates...".to_string());
    
    // Simulate check
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    updater_state.status = "up-to-date".to_string();
    updater_state.message = Some("No updates available".to_string());
    
    Ok(updater_state.clone())
}

#[tauri::command]
pub async fn updater_install(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut updater_state = state.updater_state.lock().unwrap();
    
    if updater_state.status != "ready" {
        return Err("Update is not ready to install".to_string());
    }
    
    updater_state.status = "installing".to_string();
    updater_state.message = Some("Installing update...".to_string());
    
    // In production, trigger the update installation
    // For now, just simulate
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Restart the application
    tauri::api::process::restart(&env::current_exe().unwrap());
    
    Ok(())
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
            return Err(String::from_utf8(output.stderr).unwrap_or_else(|_| "Failed to list WSL distros".to_string()));
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
pub async fn wsl_servers_install_wsl() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Open Microsoft Store to install WSL
        tauri::api::shell::open(
            &Path::new("ms-windows-store://pdp/?ProductId=9P9TQF7MRM4R"),
            None,
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("WSL is only available on Windows".to_string())
    }
}

#[tauri::command]
pub async fn wsl_servers_probe_distro(name: String) -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        // Check if specific distro exists
        let output = std::process::Command::new("wsl")
            .arg("-d")
            .arg(&name)
            .arg("--exec")
            .arg("echo")
            .arg("probed")
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            return Ok(serde_json::json!({
                "available": true,
                "distro": name,
                "message": "Distro is accessible"
            }));
        }

        return Ok(serde_json::json!({
            "available": false,
            "distro": name,
            "error": String::from_utf8(output.stderr).unwrap_or_default()
        }));
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("WSL is only available on Windows".to_string())
    }
}

#[tauri::command]
pub async fn wsl_servers_install_distro(name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        warn!("Installing WSL distro: {}", name);
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
pub async fn wsl_servers_open_terminal(name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("wsl")
            .arg("-d")
            .arg(&name)
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
pub async fn wsl_servers_add_server(config: WslServerConfig, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut wsl_servers = state.wsl_servers.lock().unwrap();
    wsl_servers.insert(config.id.clone(), config);
    Ok(())
}

#[tauri::command]
pub async fn wsl_servers_remove_server(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut wsl_servers = state.wsl_servers.lock().unwrap();
    wsl_servers.remove(&id);
    Ok(())
}

#[tauri::command]
pub async fn wsl_servers_start_server(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let wsl_servers = state.wsl_servers.lock().unwrap();
    if let Some(server) = wsl_servers.get(&id) {
        warn!("Starting WSL server: {} on port {}", server.name, server.port);
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
pub async fn create_desktop_menu(window: Window) -> Result<(), String> {
    // In Tauri, menus are created differently
    // For now, just log
    debug!("Creating desktop menu...");
    Ok(())
}

#[tauri::command]
pub async fn run_desktop_menu_action(action: String, window: Window) -> Result<(), String> {
    debug!("Running desktop menu action: {}", action);
    
    // Handle common menu actions
    match action.as_str() {
        "check-for-updates" => {
            // Trigger update check
            window.emit("menu-action", "check-for-updates").ok();
        }
        "relaunch" => {
            tauri::api::process::restart(&env::current_exe().unwrap());
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
pub async fn register_deep_link_handler(window: Window, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // In Tauri, deep links can be handled via custom protocol
    debug!("Registering deep link handler...");
    
    // Store the window reference for later use
    let mut state_lock = state.lock().unwrap();
    state_lock.main_window = Some(window);
    
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
