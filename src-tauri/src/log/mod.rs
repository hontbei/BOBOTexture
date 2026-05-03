use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Window};

use crate::config::AppState;
use crate::error::AppError;

#[derive(Debug, Serialize, Clone)]
pub struct LogEvent {
    pub level: String,
    pub source: String,
    pub message: String,
    pub timestamp: String,
}

pub fn emit(window: &Window, level: &str, source: &str, message: impl Into<String>) {
    let event = LogEvent {
        level: level.to_string(),
        source: source.to_string(),
        message: message.into(),
        timestamp: Utc::now().to_rfc3339(),
    };
    let _ = window.emit("app://log", &event);
    let app_state = window.app_handle().state::<AppState>();
    let _ = maybe_write_disk_log(window.app_handle(), app_state.inner(), &event);
}

fn maybe_write_disk_log(
    app: &AppHandle,
    state: &AppState,
    event: &LogEvent,
) -> Result<(), AppError> {
    let settings = state.load_settings()?;
    if !settings.log_to_disk {
        return Ok(());
    }

    let file_name = format!("{}.log", Utc::now().format("%Y-%m-%d"));
    let path = state.paths.log_dir.join(file_name);
    let line = format!(
        "[{}] [{}] [{}] {}\n",
        event.timestamp, event.level, event.source, event.message
    );
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(line.as_bytes())?;
    let _ = app;
    Ok(())
}

use std::io::Write;
