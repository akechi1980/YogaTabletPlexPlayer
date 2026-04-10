use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 用户可持久化的连接与播放器设置，序列化为 TOML。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub token: String,
    pub vlc_path: String,
    pub selected_library_id: String,
}

impl AppConfig {
    pub fn load_or_default(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config = toml::from_str::<Self>(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 去掉首尾空白与末尾斜杠，便于拼接 Plex API 路径。
    pub fn server_url_trimmed(&self) -> String {
        self.server_url.trim().trim_end_matches('/').to_owned()
    }

    /// 至少需要有效的服务器地址与 Token 才能请求 Plex。
    pub fn is_ready(&self) -> bool {
        !self.server_url_trimmed().is_empty() && !self.token.trim().is_empty()
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read or write config: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid config format: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("failed to serialize config: {0}")]
    TomlSer(#[from] toml::ser::Error),
}
