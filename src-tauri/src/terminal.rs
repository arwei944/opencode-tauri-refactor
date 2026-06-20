//! Terminal Integration Module
//! This module provides terminal/PTY functionality for OpenCode Tauri.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{debug, error, info, warn};
use portable_pty::{CommandBuilder, NativePtySystem, Pty, PtySize};
use serde::{Deserialize, Serialize};
use tauri::Window;

// ============================================================================
// Types
// ============================================================================

/// Terminal session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub id: String,
    pub shell: String,
    pub cwd: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub env: HashMap<String, String>,
}

/// Terminal session information
#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub id: String,
    pub pty: Pty,
    pub config: TerminalConfig,
    pub process: std::process::Child,
}

/// Terminal output event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub id: String,
    pub data: String,
}

/// Terminal size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

// ============================================================================
// Terminal Manager
// ============================================================================

/// Manages terminal sessions
pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
    next_id: Arc<Mutex<u64>>,
    pty_system: NativePtySystem,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            pty_system: portable_pty::native_pty_system(),
        }
    }

    /// Create a new terminal session
    pub async fn create_session(&self, config: TerminalConfig) -> Result<String, String> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id.to_string()
        };

        info!("Creating terminal session: {}", id);

        // Resolve shell path
        let shell = self.resolve_shell(&config.shell)?;

        // Build command
        let mut cmd = CommandBuilder::new(shell);
        
        // Set working directory
        if let Some(cwd) = &config.cwd {
            cmd.cwd(cwd);
        }

        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Set terminal size
        let size = PtySize {
            cols: config.cols,
            rows: config.rows,
        };

        // Create PTY
        let pty = self.pty_system.open_pty(&size)
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        // Spawn process
        let process = cmd.spawn_pty(pty.try_clone()?)
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        // Create session
        let session = TerminalSession {
            id: id.clone(),
            pty,
            config,
            process,
        };

        // Store session
        self.sessions.lock().unwrap().insert(id.clone(), session);

        Ok(id)
    }

    /// Destroy a terminal session
    pub async fn destroy_session(&self, id: String) -> Result<(), String> {
        info!("Destroying terminal session: {}", id);

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.remove(&id) {
            // Kill the process
            session.process.kill().await.map_err(|e| e.to_string())?;
            
            // Close PTY
            drop(session.pty);
            
            info!("Terminal session destroyed: {}", id);
        }

        Ok(())
    }

    /// Resize a terminal session
    pub async fn resize_session(&self, id: String, size: TerminalSize) -> Result<(), String> {
        debug!("Resizing terminal session {} to {}x{}", id, size.cols, size.rows);

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            let pty_size = PtySize {
                cols: size.cols,
                rows: size.rows,
            };
            
            session.pty.resize(pty_size)
                .map_err(|e| format!("Failed to resize PTY: {}", e))?;
            
            // Also send SIGWINCH to the process if on Unix
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                
                let pid = Pid::from_raw(session.process.id() as i32);
                if let Err(e) = kill(pid, Signal::SIGWINCH) {
                    warn!("Failed to send SIGWINCH: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Write data to a terminal session
    pub async fn write_to_session(&self, id: String, data: String) -> Result<(), String> {
        debug!("Writing to terminal session {}: {}", id, data);

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            session.pty.write(data.as_bytes())
                .map_err(|e| format!("Failed to write to PTY: {}", e))?;
        }

        Ok(())
    }

    /// Read from a terminal session
    pub async fn read_from_session(&self, id: String) -> Result<String, String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            let mut buffer = vec![0u8; 4096];
            let n = session.pty.read(&mut buffer)
                .map_err(|e| format!("Failed to read from PTY: {}", e))?;
            
            if n > 0 {
                buffer.truncate(n);
                return Ok(String::from_utf8_lossy(&buffer).to_string());
            }
        }

        Ok(String::new())
    }

    /// Get list of all terminal sessions
    pub async fn list_sessions(&self) -> Result<Vec<String>, String> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions.keys().cloned().collect())
    }

    /// Get terminal session info
    pub async fn get_session_info(&self, id: String) -> Result<TerminalConfig, String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            Ok(session.config.clone())
        } else {
            Err(format!("Terminal session not found: {}", id))
        }
    }

    /// Resolve shell path based on platform
    fn resolve_shell(&self, shell: &str) -> Result<String, String> {
        if !shell.is_empty() {
            return Ok(shell.to_string());
        }

        #[cfg(target_os = "windows")]
        {
            // Try to find a suitable shell
            if let Ok(path) = std::env::var("COMSPEC") {
                return Ok(path);
            }
            
            // Try PowerShell
            if std::path::Path::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe").exists() {
                return Ok("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".to_string());
            }
            
            // Fall back to cmd
            Ok("cmd.exe".to_string())
        }

        #[cfg(target_os = "macos")]
        {
            // Try to find a shell
            let shells = ["/bin/zsh", "/bin/bash", "/bin/sh"];
            for shell in shells.iter() {
                if std::path::Path::new(shell).exists() {
                    return Ok(shell.to_string());
                }
            }
            Ok("/bin/sh".to_string())
        }

        #[cfg(target_os = "linux")]
        {
            // Try to find a shell
            let user_shell = std::env::var("SHELL").ok();
            if let Some(shell) = user_shell {
                if std::path::Path::new(&shell).exists() {
                    return Ok(shell);
                }
            }
            
            // Try common shells
            let shells = ["/bin/bash", "/bin/sh", "/usr/bin/bash", "/usr/bin/sh"];
            for shell in shells.iter() {
                if std::path::Path::new(shell).exists() {
                    return Ok(shell.to_string());
                }
            }
            Ok("/bin/sh".to_string())
        }
    }
}

// ============================================================================
// Default Terminal Configuration
// ============================================================================

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            shell: String::new(),
            cwd: None,
            cols: 80,
            rows: 25,
            env: HashMap::new(),
        }
    }
}

// ============================================================================
// Terminal Commands (for Tauri)
// ============================================================================

use tauri::State;
use crate::AppState;

/// Create a new terminal session
#[tauri::command]
pub async fn terminal_create(
    config: TerminalConfig,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<String, String> {
    terminal_manager.create_session(config).await
}

/// Destroy a terminal session
#[tauri::command]
pub async fn terminal_destroy(
    id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    terminal_manager.destroy_session(id).await
}

/// Resize a terminal session
#[tauri::command]
pub async fn terminal_resize(
    id: String,
    size: TerminalSize,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    terminal_manager.resize_session(id, size).await
}

/// Write to a terminal session
#[tauri::command]
pub async fn terminal_write(
    id: String,
    data: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    terminal_manager.write_to_session(id, data).await
}

/// Read from a terminal session
#[tauri::command]
pub async fn terminal_read(
    id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<String, String> {
    terminal_manager.read_from_session(id).await
}

/// List all terminal sessions
#[tauri::command]
pub async fn terminal_list(terminal_manager: State<'_, TerminalManager>) -> Result<Vec<String>, String> {
    terminal_manager.list_sessions().await
}

/// Get terminal session info
#[tauri::command]
pub async fn terminal_get_info(
    id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<TerminalConfig, String> {
    terminal_manager.get_session_info(id).await
}
