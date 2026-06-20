//! OpenCode Tauri Desktop - Main Application
//! This is a complete reconstruction of the OpenCode Electron desktop application using Tauri.
//!
//! Migration: Electron -> Tauri
//! Benefits: Smaller bundle size, lower memory usage, faster startup, native performance

use std::env;
use std::sync::{Arc, Mutex};

use log::{debug, error, info};
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

// ============================================================================
// Modules
// ============================================================================

mod commands;
mod config;
mod terminal;

// Re-export types for convenience
pub use commands::*;
pub use config::*;
pub use terminal::*;

// ============================================================================
// Application State
// ============================================================================

// ============================================================================
// Constants
// ============================================================================

const APP_NAME: &str = "OpenCode";
const APP_VERSION: &str = "1.17.8";
const DEFAULT_WINDOW_WIDTH: f64 = 1280.0;
const DEFAULT_WINDOW_HEIGHT: f64 = 800.0;
const MIN_WINDOW_WIDTH: f64 = 800.0;
const MIN_WINDOW_HEIGHT: f64 = 600.0;
#[allow(dead_code)]
const SIDECAR_START_TIMEOUT: u64 = 60_000; // 60 秒
#[allow(dead_code)]
const SIDECAR_STOP_TIMEOUT: u64 = 6_000; // 6 秒

// ============================================================================
// Main Application Entry
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,tauri=warn"))
        .init();

    info!("{}", format_app_info());
    info!("Starting Tauri Desktop Application...");

    // Set up environment
    setup_environment();

    // Create shared application state
    let app_state = create_app_state();

    // Create terminal manager
    let terminal_manager = TerminalManager::new();

    // Build Tauri application
    tauri::Builder::default()
        // Register all plugins
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // Manage shared state
        .manage(app_state)
        .manage(terminal_manager)
        // Register all command handlers
        .invoke_handler(tauri::generate_handler![
            // ========== Window Management ==========
            commands::create_new_window,
            commands::get_window_count,
            commands::get_window_focused,
            commands::set_window_focus,
            commands::show_window,
            commands::hide_window,
            commands::close_window,
            commands::get_zoom_factor,
            commands::set_zoom_factor,
            commands::get_pinch_zoom_enabled,
            commands::set_pinch_zoom_enabled,
            commands::set_titlebar_theme,
            commands::set_background_color,
            commands::set_window_title,
            // ========== Sidecar & Server ==========
            commands::kill_sidecar,
            commands::await_initialization,
            commands::consume_initial_deep_links,
            commands::get_default_server_url,
            commands::set_default_server_url,
            // ========== App Checking ==========
            commands::check_app_exists,
            commands::resolve_app_path,
            // ========== File Pickers ==========
            commands::open_directory_picker,
            commands::open_file_picker,
            commands::save_file_picker,
            commands::read_picked_file,
            commands::release_picked_files,
            // ========== System ==========
            commands::open_link,
            commands::open_path,
            commands::read_clipboard_image,
            commands::show_notification,
            commands::relaunch,
            // ========== Display Backend ==========
            commands::get_display_backend,
            commands::set_display_backend,
            // ========== Store ==========
            commands::store_get,
            commands::store_set,
            commands::store_delete,
            commands::store_clear,
            commands::store_keys,
            commands::store_length,
            // ========== Updater ==========
            commands::updater_subscribe,
            commands::updater_unsubscribe,
            commands::updater_check,
            commands::updater_install,
            // ========== WSL (Windows) ==========
            commands::wsl_servers_get_state,
            commands::wsl_servers_probe_runtime,
            commands::wsl_servers_refresh_distros,
            commands::wsl_servers_install_wsl,
            commands::wsl_servers_probe_distro,
            commands::wsl_servers_install_distro,
            commands::wsl_servers_open_terminal,
            commands::wsl_servers_add_server,
            commands::wsl_servers_remove_server,
            commands::wsl_servers_start_server,
            // ========== Menu ==========
            commands::create_desktop_menu,
            commands::run_desktop_menu_action,
            // ========== Deep Links ==========
            commands::register_deep_link_handler,
            // ========== Terminal ==========
            terminal_create,
            terminal_destroy,
            terminal_resize,
            terminal_write,
            terminal_read,
            terminal_list,
            terminal_get_info,
            // ========== Utilities ==========
            commands::parse_markdown,
            commands::get_app_version,
            commands::get_platform,
            commands::get_arch,
            commands::export_debug_logs,
            commands::record_fatal_renderer_error,
        ])
        // Application setup
        .setup(|app| {
            info!("Setting up application...");

            // Create main window
            let window = create_main_window(app)?;

            // Store window reference in state
            let state = app.state::<AppState>();
            *state.main_window.lock().unwrap() = Some(window.clone());

            // Set up window event handlers
            setup_window_events(app, window.clone())?;

            // Spawn the OpenCode sidecar server
            spawn_sidecar_server(app.handle().clone(), window.clone());

            info!("Application setup complete");
            Ok(())
        })
        // Global window event handler
        .on_window_event(|window: &tauri::Window, event: &WindowEvent| {
            handle_global_window_event(window.clone(), event.clone());
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

// ============================================================================
// Initialization Functions
// ============================================================================

/// Create application state
fn create_app_state() -> AppState {
    AppState {
        main_window: Arc::new(Mutex::new(None)),
        sidecar_process: Arc::new(Mutex::new(None)),
        server_url: Arc::new(Mutex::new(None)),
        server_username: Arc::new(Mutex::new(None)),
        server_password: Arc::new(Mutex::new(None)),
        background_color: Arc::new(Mutex::new(None)),
        pinch_zoom_enabled: Arc::new(Mutex::new(false)),
        pending_deep_links: Arc::new(Mutex::new(Vec::new())),
        wsl_servers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        updater_state: Arc::new(Mutex::new(UpdaterState::default())),
        store_data: Arc::new(Mutex::new(std::collections::HashMap::new())),
    }
}

/// Create main window
fn create_main_window(app: &mut tauri::App) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("/".into()))
        .title(APP_NAME)
        .inner_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .min_inner_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)
        .resizable(true)
        .visible(false) // Hide initially, show after sidecar is ready
        .decorations(true)
        .build()?;

    Ok(window)
}

/// Set up window event handlers
fn setup_window_events(
    app: &mut tauri::App,
    window: WebviewWindow,
) -> Result<(), Box<dyn std::error::Error>> {
    let state_clone = (*app.state::<AppState>()).clone();
    let w = window.clone();

    window.on_window_event(move |event| {
        let window_ref = w.as_ref().window();
        handle_window_event(event.clone(), &window_ref, &state_clone);
    });

    Ok(())
}

/// Spawn the OpenCode sidecar server
fn spawn_sidecar_server(handle: tauri::AppHandle, window: WebviewWindow) {
    info!("Starting OpenCode sidecar server...");

    // In production, we would:
    // 1. Locate the OpenCode backend binary
    // 2. Spawn it as a child process
    // 3. Manage communication via IPC
    // 4. Handle lifecycle events

    // For now, use mock implementation
    tokio::spawn(async move {
        commands::spawn_sidecar(handle, window).await;
    });
}

// ============================================================================
// Event Handlers
// ============================================================================

/// Handle window-specific events
fn handle_window_event(event: WindowEvent, window: &tauri::Window, state: &AppState) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            // Prevent window from closing, hide instead (Electron-like behavior)
            api.prevent_close();
            if let Err(e) = window.hide() {
                error!("Failed to hide window on close: {}", e);
            }
        }
        WindowEvent::Destroyed => {
            info!("Window destroyed");
            // Clean up state - compare window references to clear main window
            let main_win = state.main_window.lock().unwrap();
            if let Some(ref main) = *main_win {
                if main.label() == window.label() {
                    drop(main_win);
                    *state.main_window.lock().unwrap() = None;
                }
            }
        }
        WindowEvent::Focused(is_focused) => {
            debug!("Window focus changed: {}", is_focused);
        }
        WindowEvent::Resized(size) => {
            debug!("Window resized to: {}x{}", size.width, size.height);
        }
        WindowEvent::Moved(position) => {
            debug!("Window moved to: ({}, {})", position.x, position.y);
        }
        WindowEvent::ScaleFactorChanged {
            scale_factor,
            new_inner_size,
            ..
        } => {
            debug!(
                "Scale factor changed: {}, new size: {}x{}",
                scale_factor, new_inner_size.width, new_inner_size.height
            );
        }
        _ => {}
    }
}

/// Handle global window events
fn handle_global_window_event(window: tauri::Window, event: WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        // 对所有窗口，隐藏而非关闭
        api.prevent_close();
        let _ = window.hide();
    }
}

// ============================================================================
// Environment Setup
// ============================================================================

/// Set up environment variables for the application
fn setup_environment() {
    info!("Setting up environment...");

    // OpenCode-specific environment variables
    env::set_var("OPENCODE_CLIENT", "desktop");
    env::set_var("OPENCODE_DISABLE_EMBEDDED_WEB_UI", "true");
    env::set_var("OPENCODE_EXPERIMENTAL_ICON_DISCOVERY", "true");
    env::set_var("OPENCODE_EXPERIMENTAL_FILEWATCHER", "true");

    // Platform-specific environment setup
    #[cfg(target_os = "linux")]
    {
        setup_linux_environment();
    }
}

/// Set up Linux-specific environment variables
#[cfg(target_os = "linux")]
fn setup_linux_environment() {
    // 确保 XDG 目录已设置
    if env::var("XDG_DATA_HOME").is_err() {
        if let Ok(home) = env::var("HOME") {
            env::set_var("XDG_DATA_HOME", format!("{}/.local/share", home));
        }
    }
    if env::var("XDG_CONFIG_HOME").is_err() {
        if let Ok(home) = env::var("HOME") {
            env::set_var("XDG_CONFIG_HOME", format!("{}/.config", home));
        }
    }
    if env::var("XDG_CACHE_HOME").is_err() {
        if let Ok(home) = env::var("HOME") {
            env::set_var("XDG_CACHE_HOME", format!("{}/.cache", home));
        }
    }
    if env::var("XDG_STATE_HOME").is_err() {
        if let Ok(home) = env::var("HOME") {
            env::set_var("XDG_STATE_HOME", format!("{}/.local/state", home));
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Format application information
fn format_app_info() -> String {
    format!(
        "{} v{} | Platform: {} | Arch: {}",
        APP_NAME,
        APP_VERSION,
        env::consts::OS,
        env::consts::ARCH
    )
}

/// Get application user data directory
#[allow(dead_code)]
fn get_user_data_dir() -> std::path::PathBuf {
    config::get_app_data_dir()
}

/// Get application config directory
#[allow(dead_code)]
fn get_config_dir() -> std::path::PathBuf {
    config::get_app_config_dir()
}

/// Get application cache directory
#[allow(dead_code)]
fn get_cache_dir() -> std::path::PathBuf {
    config::get_app_cache_dir()
}
