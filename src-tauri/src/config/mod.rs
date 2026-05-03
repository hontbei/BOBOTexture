use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::AppError;

const APP_NAME: &str = "BOBOTexture";
const SETTINGS_FILE: &str = "config.json";
const LOG_DIR: &str = "logs";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub language: String,
    pub launch_animation: bool,
    pub particle_level: String,
    pub window_width: f64,
    pub window_height: f64,
    pub log_to_disk: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_project_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "zh".to_string(),
            launch_animation: true,
            particle_level: "high".to_string(),
            window_width: 1280.0,
            window_height: 800.0,
            log_to_disk: false,
            last_project_path: None,
        }
    }
}

#[derive(Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub settings: PathBuf,
    pub log_dir: PathBuf,
}

pub struct AppState {
    pub paths: AppPaths,
    pub settings: Mutex<AppSettings>,
}

impl AppState {
    pub fn bootstrap(app: &AppHandle) -> Result<Self, AppError> {
        let root = resolve_app_dir(app)?;
        fs::create_dir_all(&root)?;
        let log_dir = root.join(LOG_DIR);
        fs::create_dir_all(&log_dir)?;

        let settings_path = root.join(SETTINGS_FILE);
        if !settings_path.exists() {
            write_json(&settings_path, &AppSettings::default())?;
        }

        let settings = read_json::<AppSettings>(&settings_path)?.unwrap_or_default();

        Ok(Self {
            paths: AppPaths {
                root,
                settings: settings_path,
                log_dir,
            },
            settings: Mutex::new(settings),
        })
    }

    pub fn load_settings(&self) -> Result<AppSettings, AppError> {
        read_json::<AppSettings>(&self.paths.settings)?.ok_or_else(|| {
            AppError::new("settings_missing", "settings file is missing or invalid")
        })
    }

    pub fn save_settings(&self, settings: AppSettings) -> Result<(), AppError> {
        write_json(&self.paths.settings, &settings)?;
        let mut guard = self
            .settings
            .lock()
            .map_err(|_| AppError::new("settings_lock", "cannot lock settings state"))?;
        *guard = settings;
        Ok(())
    }
}

fn resolve_app_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    if let Some(path) = app.path().app_config_dir().ok() {
        return Ok(path.join(APP_NAME));
    }

    let base = dirs::config_dir().ok_or_else(|| {
        AppError::new("config_dir_missing", "unable to resolve configuration directory")
    })?;
    Ok(base.join(APP_NAME))
}

fn read_json<T>(path: &Path) -> Result<Option<T>, AppError>
where
    T: for<'de> Deserialize<'de>,
{
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn write_json<T>(path: &Path, value: &T) -> Result<(), AppError>
where
    T: Serialize,
{
    let content = serde_json::to_string_pretty(value)?;
    fs::write(path, content)?;
    Ok(())
}
