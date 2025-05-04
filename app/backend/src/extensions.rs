use std::{collections::HashMap, path::PathBuf};

use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{to_tauri_error, AppState};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionManifest {
    pub name: String,
    pub display_name: String,
    pub version: Version,
    pub description: String,
    pub latest_url: String,
}

#[derive(Deserialize)]
pub struct ExtensionLatest {
    pub version: Version,
    pub assets: HashMap<String, Asset>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub url: String,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionInfo {
    pub manifest: ExtensionManifest,
    pub icon_path: PathBuf,
    pub enabled: bool,
}

/// Returns the extension info of all extensions
pub fn info_extensions(app_state: State<'_, AppState>) -> tauri::Result<Vec<ExtensionInfo>> {
    let extensions_path = app_state.extensions_path.clone();
    let config = app_state.config.read().map_err(to_tauri_error)?.clone();
    let enabled = config.enabled;
    let ordered = config.ordered;

    let dirs = extensions_path.read_dir()?;

    let mut extensions = Vec::new();
    let mut paths: Vec<PathBuf> = dirs.filter_map(|e| Some(e.ok()?.path())).collect();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            let manifest_path = extensions_path.join(&path).join("manifest.json");

            let manifest_data = std::fs::read_to_string(&manifest_path)?;
            let manifest: ExtensionManifest = serde_json::from_str(&manifest_data)?;

            let icon_path = extensions_path.join(&path).join("icon.svg");

            extensions.push(ExtensionInfo {
                manifest,
                icon_path,
                enabled: enabled.contains(
                    &path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                ),
            });
        }
    }

    sort_by_order(&mut extensions, &ordered);

    Ok(extensions)
}

fn sort_by_order(v: &mut [ExtensionInfo], ordered: &[String]) {
    let rank: HashMap<&str, usize> = ordered
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();

    v.sort_by_key(|i| {
        let r = rank
            .get(i.manifest.name.as_str())
            .cloned()
            .unwrap_or(usize::MAX);
        (r, i.manifest.name.clone())
    });
}

/// Emits an extension update to the main window.
pub fn emit_extensions_update(app: &AppHandle) -> tauri::Result<()> {
    let app_state = app.state::<AppState>();
    let extensions = info_extensions(app_state)?;

    if let Some(main_window) = app.get_webview_window("main") {
        main_window.emit("update-extensions", extensions.clone())?;
    }

    Ok(())
}
