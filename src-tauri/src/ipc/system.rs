use std::fs;
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, Window};
use walkdir::WalkDir;

use crate::config::{AppSettings, AppState};
use crate::error::AppError;
use crate::log;

#[derive(Debug, Serialize)]
pub struct BootstrapPayload {
    pub settings: AppSettings,
    pub config_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportLogsRequest {
    pub output_path: String,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CollectFilesRequest {
    pub inputs: Vec<String>,
    pub recursive: bool,
}

#[derive(Debug, Deserialize)]
pub struct WriteTextFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified_ms: i64,
}

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> Result<BootstrapPayload, AppError> {
    let settings = state.load_settings()?;
    Ok(BootstrapPayload {
        settings,
        config_dir: state.paths.root.display().to_string(),
    })
}

#[tauri::command]
pub fn save_settings(
    window: Window,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), AppError> {
    state.save_settings(settings)?;
    log::emit(&window, "info", "system", "settings updated");
    Ok(())
}

#[tauri::command]
pub fn export_logs(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ExportLogsRequest,
) -> Result<(), AppError> {
    if let Some(content) = request.content {
        fs::write(request.output_path, content)?;
        let _ = app;
        return Ok(());
    }

    let mut buffer = String::new();
    for entry in fs::read_dir(&state.paths.log_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("log") {
            buffer.push_str(&fs::read_to_string(path)?);
        }
    }
    fs::write(request.output_path, buffer)?;
    let _ = app;
    Ok(())
}

#[tauri::command]
pub fn collect_files(request: CollectFilesRequest) -> Result<Vec<FileEntry>, AppError> {
    let mut items = Vec::new();

    for input in request.inputs {
        let path = std::path::PathBuf::from(&input);
        if path.is_file() {
            items.push(build_file_entry(&path)?);
            continue;
        }

        let walker = if request.recursive {
            WalkDir::new(&path)
        } else {
            WalkDir::new(&path).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(Result::ok) {
            let candidate = entry.path();
            if candidate.is_file() {
                items.push(build_file_entry(candidate)?);
            }
        }
    }

    items.sort_by(|left, right| left.path.cmp(&right.path));
    items.dedup_by(|left, right| left.path == right.path);
    Ok(items)
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), AppError> {
    let target = std::path::PathBuf::from(&path);
    if !target.exists() {
        return Err(AppError::new(
            "open_in_explorer_not_found",
            format!("path does not exist: {}", path),
        ));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(&path).spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(&path).spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(&path).spawn()?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(AppError::new(
            "open_in_explorer_unsupported",
            "open_in_explorer is not supported on this platform",
        ))
    }
}

fn build_file_entry(path: &std::path::Path) -> Result<FileEntry, AppError> {
    let metadata = fs::metadata(path)?;
    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default();

    Ok(FileEntry {
        path: path.display().to_string(),
        name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string(),
        size: metadata.len(),
        modified_ms,
    })
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, AppError> {
    fs::read_to_string(&path).map_err(|e| AppError::new("read_file", format!("{}: {e}", path)))
}

#[tauri::command]
pub fn write_text_file(request: WriteTextFileRequest) -> Result<(), AppError> {
    if let Some(parent) = std::path::Path::new(&request.path).parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::new("write_file", format!("{}: {e}", request.path)))?;
    }
    fs::write(&request.path, &request.content).map_err(|e| AppError::new("write_file", format!("{}: {e}", request.path)))
}

#[tauri::command]
pub fn path_exists(path: String) -> Result<bool, AppError> {
    Ok(std::path::Path::new(&path).exists())
}
