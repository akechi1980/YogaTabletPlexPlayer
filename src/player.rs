use std::path::Path;
use std::process::Command;

use thiserror::Error;

/// 以单个媒体 URL 为参数启动 VLC（不等待进程结束）。
pub fn launch_vlc(
    vlc_path: &str,
    media_url: &str,
    media_title: &str,
    start_offset_ms: Option<u64>,
) -> Result<(), PlayerError> {
    let trimmed = vlc_path.trim();
    if trimmed.is_empty() {
        return Err(PlayerError::MissingPath);
    }

    if !Path::new(trimmed).exists() {
        return Err(PlayerError::MissingExecutable(trimmed.to_owned()));
    }

    let mut command = Command::new(trimmed);
    if !media_title.trim().is_empty() {
        command
            .arg("--meta-title")
            .arg(media_title)
            .arg("--input-title-format")
            .arg(media_title);
    }

    if let Some(offset_ms) = start_offset_ms.filter(|offset| *offset >= 1_000) {
        let seconds = offset_ms as f64 / 1_000.0;
        command.arg("--start-time").arg(format!("{seconds:.3}"));
    }

    command
        .arg(media_url)
        .spawn()
        .map_err(PlayerError::LaunchFailed)?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("VLC path is not configured")]
    MissingPath,
    #[error("VLC executable not found: {0}")]
    MissingExecutable(String),
    #[error("failed to launch VLC: {0}")]
    LaunchFailed(std::io::Error),
}
