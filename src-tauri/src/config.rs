//! Application Configuration
//! This module handles application configuration and settings management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ============================================================================
// Configuration Types
// ============================================================================

/// Application configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub backend: Option<String>, // "wayland", "x11" 或 "auto"
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
#[derive(Default)]
pub struct ConfigManager {
    config: Arc<Mutex<AppConfig>>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self::default()
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

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // ---- AppConfig 默认值测试 ----

    #[test]
    fn test_app_config_default_values() {
        let cfg = AppConfig::default();
        // server 字段是 ServerConfig::default()
        assert_eq!(cfg.server.hostname, "127.0.0.1");
        assert_eq!(cfg.server.port, 0);
        assert_eq!(cfg.server.username, "opencode");
        // window 默认值
        assert_eq!(cfg.window.width, 1280.0);
        assert_eq!(cfg.window.height, 800.0);
        assert_eq!(cfg.window.min_width, 800.0);
        assert_eq!(cfg.window.min_height, 600.0);
        assert!(cfg.window.resizable);
        assert!(cfg.window.decorations);
        assert!(!cfg.window.pinch_zoom_enabled);
        assert_eq!(cfg.window.titlebar_theme, "dark");
        // features 默认值
        assert!(cfg.features.wsl_integration);
        assert!(cfg.features.auto_updater);
        assert!(cfg.features.multiple_windows);
        assert!(cfg.features.deep_links);
        assert!(!cfg.features.analytics);
        // wsl 默认值
        assert!(cfg.wsl.enabled);
        assert!(cfg.wsl.distros.is_empty());
        // updater 默认值
        assert!(cfg.updater.enabled);
        assert_eq!(cfg.updater.check_interval, 60);
        assert_eq!(cfg.updater.channel, "stable");
        // display 默认值
        assert!(cfg.display.backend.is_none());
    }

    // ---- 各子结构体默认值 ----

    #[test]
    fn test_server_config_default() {
        let s = ServerConfig::default();
        assert!(s.default_url.is_none());
        assert_eq!(s.hostname, "127.0.0.1");
        assert_eq!(s.port, 0);
        assert_eq!(s.username, "opencode");
        assert!(s.password.is_empty());
    }

    #[test]
    fn test_window_config_default() {
        let w = WindowConfig::default();
        assert_eq!(w.width, 1280.0);
        assert_eq!(w.height, 800.0);
        assert!(w.resizable);
        assert!(w.decorations);
        assert!(w.background_color.is_none());
    }

    #[test]
    fn test_feature_flags_default() {
        let f = FeatureFlags::default();
        assert!(f.wsl_integration);
        assert!(f.auto_updater);
        assert!(f.multiple_windows);
        assert!(f.deep_links);
        assert!(!f.analytics);
    }

    // ---- ConfigManager 保存/加载/更新 ----

    #[test]
    fn test_config_manager_save_and_load() {
        // 使用临时目录
        let mut tmp = env::temp_dir();
        tmp.push(format!("opencode-test-{}.json", std::process::id()));
        // 确保文件不存在
        let _ = std::fs::remove_file(&tmp);

        // 保存：先 load 不存在的文件，会触发默认值并保存
        let mut mgr = ConfigManager::new();
        mgr.load(tmp.clone())
            .expect("load should succeed with defaults");
        assert!(tmp.exists(), "配置文件应当被创建");

        // 加载回来
        let mut mgr2 = ConfigManager::new();
        mgr2.load(tmp.clone()).expect("load should succeed");
        let cfg = mgr2.get();
        assert_eq!(cfg.server.hostname, "127.0.0.1");
        assert_eq!(cfg.window.width, 1280.0);

        // 清理
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_config_manager_update_and_persist() {
        let mut tmp = env::temp_dir();
        tmp.push(format!("opencode-update-test-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&tmp);

        let mgr = ConfigManager::new();
        mgr.load(tmp.clone()).expect("load");

        // 修改并保存
        mgr.update(|c| {
            c.window.width = 1600.0;
            c.server.port = 8080;
        })
        .expect("update");

        // 重新加载验证持久化
        let mgr2 = ConfigManager::new();
        mgr2.load(tmp.clone()).expect("reload");
        let cfg = mgr2.get();
        assert_eq!(cfg.window.width, 1600.0);
        assert_eq!(cfg.server.port, 8080);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_config_manager_load_invalid_json() {
        let mut tmp = env::temp_dir();
        tmp.push(format!("opencode-invalid-{}.json", std::process::id()));
        std::fs::write(&tmp, "{ 这不是合法 JSON }").unwrap();

        let mut mgr = ConfigManager::new();
        let res = mgr.load(tmp.clone());
        assert!(res.is_err(), "非法 JSON 应当报错");

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_config_manager_get_sub_configs() {
        let mgr = ConfigManager::new();
        mgr.load(env::temp_dir().join("dummy.json")).expect("load");

        let server = mgr.get_server_config();
        assert_eq!(server.hostname, "127.0.0.1");

        let window = mgr.get_window_config();
        assert_eq!(window.width, 1280.0);

        let display = mgr.get_display_config();
        assert!(display.backend.is_none());

        let features = mgr.get_feature_flags();
        assert!(features.deep_links);
    }

    // ---- 目录解析函数 ----

    #[test]
    fn test_app_dirs_return_pathbuf() {
        // 不依赖具体平台路径，只确认返回的是 PathBuf 且非空
        let d = get_app_data_dir();
        assert!(!d.as_os_str().is_empty());
        let c = get_app_config_dir();
        assert!(!c.as_os_str().is_empty());
        let k = get_app_cache_dir();
        assert!(!k.as_os_str().is_empty());
    }

    #[test]
    fn test_app_dirs_contain_app_name() {
        // 跨平台：路径应当包含 OpenCode 或 opencode
        let d = get_app_data_dir();
        let s = d.to_string_lossy().to_lowercase();
        assert!(
            s.contains("opencode"),
            "数据目录路径应包含 'opencode': {}",
            s
        );
    }

    // ---- 序列化往返 ----

    #[test]
    fn test_app_config_serde_roundtrip() {
        let original = AppConfig::default();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: AppConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.server.hostname, original.server.hostname);
        assert_eq!(restored.window.width, original.window.width);
        assert_eq!(restored.features.deep_links, original.features.deep_links);
        assert_eq!(
            restored.updater.check_interval,
            original.updater.check_interval
        );
    }
}
