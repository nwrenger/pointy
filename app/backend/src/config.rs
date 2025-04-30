use std::{fs, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{to_tauri_error, AppState};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub autostart: bool,
    pub shortcut: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            shortcut: String::from("CommandOrControl+Shift+Space"),
        }
    }
}

/// Loads the app config
pub fn load_app_config(path: &PathBuf) -> tauri::Result<AppConfig> {
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

/// Should be called when the config changed.
pub fn update_config(app: &AppHandle) -> tauri::Result<()> {
    let app_state: State<'_, AppState> = app.state();

    let old_app_config = app_state.app_config.read().map_err(to_tauri_error)?;
    let new_app_config = load_app_config(&app_state.config_path)?;

    // Remove old shortcut
    let old_shortcut = Shortcut::from_str(&old_app_config.shortcut).map_err(to_tauri_error)?;
    app.global_shortcut()
        .unregister(old_shortcut)
        .map_err(to_tauri_error)?;

    drop(old_app_config);

    // Add new shortcut
    let new_shortcut = Shortcut::from_str(&new_app_config.shortcut).map_err(to_tauri_error)?;
    app.global_shortcut()
        .register(new_shortcut)
        .map_err(to_tauri_error)?;

    // Configure autostart
    let autostart_manager = app.autolaunch();
    if new_app_config.autostart {
        autostart_manager.enable().map_err(to_tauri_error)?;
    } else {
        autostart_manager.disable().map_err(to_tauri_error)?;
    }

    let mut w = app_state.app_config.write().map_err(to_tauri_error)?;
    *w = new_app_config;

    Ok(())
}
