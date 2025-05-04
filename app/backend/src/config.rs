use std::{fs, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{extensions::emit_extensions_update, to_tauri_error, AppState};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub autolaunch: bool,
    pub shortcut: String,
    pub enabled: Vec<String>,
    pub ordered: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            autolaunch: false,
            shortcut: String::from("CommandOrControl+Shift+Space"),
            enabled: vec![],
            ordered: vec![],
        }
    }
}

/// Loads the app config from a path
pub fn load_config(path: &PathBuf) -> tauri::Result<Config> {
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

/// Fetches the current config from the state
#[tauri::command]
pub fn get_config(app_state: State<'_, AppState>) -> Result<Config, String> {
    let config = app_state.config.read().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Changes stored config and applies changes of the config to the app
#[tauri::command]
pub fn change_config(
    new_config: Config,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> tauri::Result<Config> {
    let old_config = app_state.config.read().map_err(to_tauri_error)?;

    // Check if shortcut has changed
    if old_config.shortcut != new_config.shortcut {
        // Remove old shortcut
        let old_shortcut = Shortcut::from_str(&old_config.shortcut).map_err(to_tauri_error)?;
        app.global_shortcut()
            .unregister(old_shortcut)
            .map_err(to_tauri_error)?;

        // Add new shortcut
        let new_shortcut = Shortcut::from_str(&new_config.shortcut).map_err(to_tauri_error)?;
        app.global_shortcut()
            .register(new_shortcut)
            .map_err(to_tauri_error)?;
    }

    if new_config.autolaunch != old_config.autolaunch {
        set_autolaunch(&new_config, &app).map_err(to_tauri_error)?;
    }

    let old_config_clone = old_config.clone();

    drop(old_config);

    // Save to app state
    let mut config = app_state.config.write().map_err(to_tauri_error)?;
    *config = new_config.clone();

    drop(config);

    // Check if enabled changed, if so emit an extension update
    if new_config.enabled != old_config_clone.enabled
        || new_config.ordered != old_config_clone.ordered
    {
        emit_extensions_update(&app)?;
    }

    Ok(new_config)
}

pub fn set_autolaunch(
    config: &Config,
    app: &AppHandle,
) -> Result<(), tauri_plugin_autostart::Error> {
    let autostart_manager = app.autolaunch();
    if config.autolaunch {
        if !autostart_manager.is_enabled()? {
            autostart_manager.enable()?;
        }
    } else if autostart_manager.is_enabled()? {
        autostart_manager.disable()?;
    }

    Ok(())
}

/// Persists the app config by saving it to the disk
pub fn persist_config(app: &AppHandle) -> tauri::Result<()> {
    let app_state: State<'_, AppState> = app.state();
    let config = app_state.config.read().map_err(to_tauri_error)?;

    fs::write(&app_state.config_path, serde_json::to_string(&*config)?)?;

    Ok(())
}
