use std::{fs, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{error, extensions::emit_extensions_update, AppState};

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
            shortcut: {
                #[cfg(target_os = "macos")]
                {
                    String::from("Command+Shift+Space")
                }
                #[cfg(not(target_os = "macos"))]
                {
                    String::from("Control+Shift+Space")
                }
            },
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
pub fn get_config(app_state: State<'_, AppState>) -> error::Result<Config> {
    let config = app_state.config.read()?;
    Ok(config.clone())
}

/// Changes stored config and saves it to disk. Also applies changes of the config to the app
#[tauri::command]
pub fn change_config(
    new_config: Config,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> error::Result<Config> {
    let mut config = app_state.config.write()?;
    let old_config = config.clone();

    // Only update when the shortcut string actually changed
    if old_config.shortcut != new_config.shortcut {
        // First, try registering the new shortcut.
        // We register before unregistering the old one so that
        // if the new shortcut is invalid or conflicts, the old shortcut
        // remains active and the app stays functional.
        let new_shortcut = Shortcut::from_str(&new_config.shortcut)?;
        app.global_shortcut().register(new_shortcut)?;

        // Now that the new shortcut is live, remove the old registration
        let old_shortcut = Shortcut::from_str(&old_config.shortcut)?;
        app.global_shortcut().unregister(old_shortcut)?;
    }

    // Only update when autolaunch actually changed
    if new_config.autolaunch != old_config.autolaunch {
        set_autolaunch(&new_config, &app)?;
    }

    // Save to app state
    *config = new_config.clone();
    drop(config);

    // Persist config
    fs::write(&app_state.config_path, serde_json::to_string(&new_config)?)?;

    // Emit an extension update if enabled or ordered changed
    if new_config.enabled != old_config.enabled || new_config.ordered != old_config.ordered {
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
