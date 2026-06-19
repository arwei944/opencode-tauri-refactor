//! Application Configuration
//! This module handles application configuration and settings management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ============================================================================
// Configuration Types
// ============================================================================

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Window configuration
    pub window: WindowConfig,
    /// Display configuration
    pub display: DisplayConfig,
    /// Feature flags
    pub features: FeatureFlags,
    /// WSL configuration (Windows only)
    pub wsl: WslConfig,
    /// Update configuration
    pub updater: UpdaterConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub default_url: Option<String>,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f64,
    pub height: f64,
    pub min_width: f64,
    pub min_height: f64,
    pub resizable: bool,
    pub decorations: bool,
    pub background_color: Option<String>,
    pub pinch_zoom_enabled: bool,
    pub titlebar_theme: String,
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub backend: Option<String>, // "wayland", "x11", or "auto"
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub wsl_integration: bool,
    pub auto_updater: bool,
    pub multiple_windows: bool,
    pub deep_links: bool,
    pub analytics: bool,
}

/// WSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslConfig {
    pub enabled: bool,
    pub distros: Vec<WslDistroConfig>,
}

/// WSL distro configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslDistroConfig {
    pub id: String,
    pub name: String,
    pub auto_start: bool,
}

/// Updater configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterConfig {
    pub enabled: bool,
    pub check_interval: u64, // in minutes
    pub channel: String,     // "stable", "beta", "dev"
}

// ============================================================================
// Default Configuration
// ============================================================================

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            window: WindowConfig::default(),
            display: DisplayConfig::default(),
            features: FeatureFlags::default(),
            wsl: WslConfig::default(),
            updater: UpdaterConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            default_url: None,
            hostname: "127.0.0.1".to_string(),
            port: 0, // 0 means let OS choose
            username: "opencode".to_string(),
            password: "".to_string(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 800.0,
            min_width: 800.0,
            min_height: 600.0,
            resizable: true,
            decorations: true,
            background_color: None,
            pinch_zoom_enabled: false,
            titlebar_theme: "dark".to_string(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self { backend: None }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            wsl_integration: true,
            auto_updater: true,
            multiple_windows: true,
            deep_links: true,
            analytics: false,
        }
    }
}

impl Default for WslConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            distros: Vec::new(),
        }
    }
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: 60, // 1 hour
            channel: "stable".to_string(),
        }
    }
}

// ============================================================================
// Configuration Manager
// ============================================================================

/// Manages application configuration
pub struct ConfigManager {
    config: Arc<Mutex<AppConfig>>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AppConfig::default())),
            config_path: PathBuf::new(),
        }
    }

    /// Load configuration from file
    pub fn load(&mut self, path: PathBuf) -> Result<(), String> {
        self.config_path = path;
        
        // Try to read config file
        match std::fs::read_to_string(&self.config_path) {
            Ok(content) => {
                let config: AppConfig = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse config: {}", e))?;
                *self.config.lock().unwrap() = config;
                Ok(())
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Config file doesn't exist, use defaults
                    self.save()?;
                    Ok(())
                } else {
                    Err(format!("Failed to read config: {}", e))
                }
            }
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), String> {
        if self.config_path.as_os_str().is_empty() {
            return Err("Config path not set".to_string());
        }
        
        let config = self.config.lock().unwrap();
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        std::fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        
        Ok(())
    }

    /// Get current configuration
    pub fn get(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    /// Update configuration
    pub fn update<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.lock().unwrap();
        f(&mut config);
        self.save()?;
        Ok(())
    }

    /// Get server configuration
    pub fn get_server_config(&self) -> ServerConfig {
        self.config.lock().unwrap().server.clone()
    }

    /// Get window configuration
    pub fn get_window_config(&self) -> WindowConfig {
        self.config.lock().unwrap().window.clone()
    }

    /// Get display configuration
    pub fn get_display_config(&self) -> DisplayConfig {
        self.config.lock().unwrap().display.clone()
    }

    /// Get feature flags
    pub fn get_feature_flags(&self) -> FeatureFlags {
        self.config.lock().unwrap().features.clone()
    }
}

// ============================================================================
// Environment Configuration
// ============================================================================

/// Get application data directory
pub fn get_app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("OpenCode")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("OpenCode")
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::data_dir().unwrap_or_else(|| PathBuf::from("~")))
            .join("opencode")
    }
}

/// Get application config directory
pub fn get_app_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("OpenCode")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("OpenCode")
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from("~")))
            .join("opencode")
    }
}

/// Get application cache directory
pub fn get_app_cache_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("OpenCode")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("OpenCode")
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::cache_dir().unwrap_or_else(|| PathBuf::from("~")))
            .join("opencode")
    }
}
