//! 终端集成模块
//! 为 OpenCode Tauri 提供终端/PTY 功能。

use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{Arc, Mutex},
};

use log::{debug, info};
use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, PtySystem, SlavePty};
use serde::{Deserialize, Serialize};

// ============================================================================
// 类型定义
// ============================================================================

/// 终端会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub id: String,
    pub shell: String,
    pub cwd: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub env: HashMap<String, String>,
}

/// 终端会话信息
pub struct TerminalSession {
    pub id: String,
    pub master: Box<dyn MasterPty + Send>,
    pub slave: Box<dyn SlavePty + Send>,
    pub config: TerminalConfig,
    pub child_killer: Box<dyn ChildKiller + Send + Sync>,
}

impl std::fmt::Debug for TerminalSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalSession")
            .field("id", &self.id)
            .field("config", &self.config)
            .finish()
    }
}

/// 终端输出事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub id: String,
    pub data: String,
}

/// 终端尺寸
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

// ============================================================================
// 终端管理器
// ============================================================================

/// 管理终端会话
pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
    next_id: Arc<Mutex<u64>>,
    pty_system: portable_pty::PtySystem,
}

impl TerminalManager {
    /// 创建新的终端管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            pty_system: portable_pty::native_pty_system(),
        }
    }

    /// 创建新的终端会话
    pub async fn create_session(&self, config: TerminalConfig) -> Result<String, String> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id.to_string()
        };

        info!("创建终端会话: {}", id);

        // 解析 shell 路径
        let shell = self.resolve_shell(&config.shell)?;

        // 构建命令
        let mut cmd = CommandBuilder::new(shell);

        // 设置工作目录
        if let Some(cwd) = &config.cwd {
            cmd.cwd(cwd);
        }

        // 设置环境变量
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // 设置终端尺寸
        let size = PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        // 创建 PTY 对
        let pair = self
            .pty_system
            .open_pty(&size)
            .map_err(|e| format!("打开 PTY 失败: {}", e))?;

        // 在从 PTY 中生成进程
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("生成进程失败: {}", e))?;

        // 创建会话
        let session = TerminalSession {
            id: id.clone(),
            master: pair.master,
            slave: pair.slave,
            config,
            child_killer: child,
        };

        // 存储会话
        self.sessions.lock().unwrap().insert(id.clone(), session);

        Ok(id)
    }

    /// 销毁终端会话
    pub async fn destroy_session(&self, id: String) -> Result<(), String> {
        info!("销毁终端会话: {}", id);

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(&id) {
            // 杀死进程（同步）
            session.child_killer.kill().map_err(|e| e.to_string())?;
            // 关闭 PTY（drop 自动处理）

            info!("终端会话已销毁: {}", id);
        }

        Ok(())
    }

    /// 调整终端会话尺寸
    pub async fn resize_session(&self, id: String, size: TerminalSize) -> Result<(), String> {
        debug!("调整终端会话尺寸 {} 为 {}x{}", id, size.cols, size.rows);

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            let pty_size = PtySize {
                rows: size.rows,
                cols: size.cols,
                pixel_width: 0,
                pixel_height: 0,
            };

            session
                .master
                .resize(pty_size)
                .map_err(|e| format!("调整 PTY 尺寸失败: {}", e))?;
        }

        Ok(())
    }

    /// 向终端会话写入数据
    pub async fn write_to_session(&self, id: String, data: String) -> Result<(), String> {
        debug!("向终端会话 {} 写入数据: {}", id, data);

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            let mut writer = session
                .master
                .take_writer()
                .map_err(|e| format!("获取写入器失败: {}", e))?;

            writer
                .write_all(data.as_bytes())
                .map_err(|e| format!("写入 PTY 失败: {}", e))?;
            writer.flush().map_err(|e| format!("刷新失败: {}", e))?;
        }

        Ok(())
    }

    /// 从终端会话读取数据
    pub async fn read_from_session(&self, id: String) -> Result<String, String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            let mut reader = session
                .master
                .try_clone_reader()
                .map_err(|e| format!("克隆读取器失败: {}", e))?;

            let mut buffer = vec![0u8; 4096];
            let n = reader
                .read(&mut buffer)
                .map_err(|e| format!("读取 PTY 失败: {}", e))?;

            if n > 0 {
                buffer.truncate(n);
                return Ok(String::from_utf8_lossy(&buffer).to_string());
            }
        }

        Ok(String::new())
    }

    /// 获取所有终端会话列表
    pub async fn list_sessions(&self) -> Result<Vec<String>, String> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions.keys().cloned().collect())
    }

    /// 获取终端会话信息
    pub async fn get_session_info(&self, id: String) -> Result<TerminalConfig, String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&id) {
            Ok(session.config.clone())
        } else {
            Err(format!("未找到终端会话: {}", id))
        }
    }

    /// 基于平台解析 shell 路径
    fn resolve_shell(&self, shell: &str) -> Result<String, String> {
        if !shell.is_empty() {
            return Ok(shell.to_string());
        }

        #[cfg(target_os = "windows")]
        {
            // 尝试找到合适的 shell
            if let Ok(path) = std::env::var("COMSPEC") {
                return Ok(path);
            }

            // 尝试 PowerShell
            if std::path::Path::new(
                "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
            )
            .exists()
            {
                return Ok(
                    "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".to_string(),
                );
            }

            // 回退到 cmd
            Ok("cmd.exe".to_string())
        }

        #[cfg(target_os = "macos")]
        {
            // 尝试找到 shell
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
            // 尝试找到 shell
            let user_shell = std::env::var("SHELL").ok();
            if let Some(shell) = user_shell {
                if std::path::Path::new(&shell).exists() {
                    return Ok(shell);
                }
            }

            // 尝试常见的 shell
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
// 默认终端配置
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
// Tauri 终端命令
// ============================================================================

use tauri::State;

/// 创建新终端会话
#[tauri::command]
pub async fn terminal_create(
    config: TerminalConfig,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<String, String> {
    terminal_manager.create_session(config).await
}

/// 销毁终端会话
#[tauri::command]
pub async fn terminal_destroy(
    id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    terminal_manager.destroy_session(id).await
}

/// 调整终端会话尺寸
#[tauri::command]
pub async fn terminal_resize(
    id: String,
    size: TerminalSize,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    terminal_manager.resize_session(id, size).await
}

/// 向终端写入数据
#[tauri::command]
pub async fn terminal_write(
    id: String,
    data: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    terminal_manager.write_to_session(id, data).await
}

/// 从终端读取数据
#[tauri::command]
pub async fn terminal_read(
    id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<String, String> {
    terminal_manager.read_from_session(id).await
}

/// 列出所有终端会话
#[tauri::command]
pub async fn terminal_list(
    terminal_manager: State<'_, TerminalManager>,
) -> Result<Vec<String>, String> {
    terminal_manager.list_sessions().await
}

/// 获取终端会话信息
#[tauri::command]
pub async fn terminal_get_info(
    id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<TerminalConfig, String> {
    terminal_manager.get_session_info(id).await
}
