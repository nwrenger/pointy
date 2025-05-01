use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};

use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
pub struct ExtensionManifest {
    pub name: String,
    pub display_name: String,
    pub version: Version,
    pub description: String,
    pub manifest_url: String,
}

#[derive(Deserialize)]
pub struct ExtensionUpdate {
    pub version: Version,
    pub assets: HashMap<String, Asset>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub url: String,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExtensionInfo {
    pub manifest: ExtensionManifest,
    pub icon_path: PathBuf,
    pub enabled: bool,
}

pub fn info_extensions(app_state: State<'_, AppState>) -> tauri::Result<Vec<ExtensionInfo>> {
    let extensions_path = app_state.extensions_path.clone();

    let enabled_path = extensions_path.join("enabled.json");
    let data = std::fs::read_to_string(&enabled_path)?;
    let enabled: Vec<PathBuf> = serde_json::from_str(&data)?;

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
                enabled: enabled.contains(&PathBuf::from(path.file_name().unwrap_or_default())),
            });
        }
    }

    Ok(extensions)
}

/// Toggles an `extension`
pub fn toggle_extension(key: String, state: State<'_, AppState>) -> tauri::Result<()> {
    let path = state.extensions_path.join("enabled.json");

    let mut list = load_enabled_extensions(&path)?;
    if list.contains(&key) {
        list.retain(|i| i != &key);
    } else {
        list.push(key);
    }

    save_enabled_extensions(&path, &list)
}

/// Saves the enabled extensions
pub fn save_enabled_extensions(path: &PathBuf, extensions: &[String]) -> tauri::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    Ok(serde_json::to_writer(writer, extensions)?)
}

/// Gets the enabled extensions
pub fn load_enabled_extensions(path: &PathBuf) -> tauri::Result<Vec<String>> {
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}
