use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
};

use flate2::read::GzDecoder;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::Archive;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    error::{self, Error},
    AppState,
};

pub const EXTENSIONS_URL: &str =
    "https://raw.githubusercontent.com/nwrenger/pointy-extensions/refs/heads/main/extensions.json";

/// Extension metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: Version,
    pub description: String,
    pub latest_url: String,
}

/// Extension download information
#[derive(Serialize, Deserialize)]
pub struct AvailableExtension {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub latest_url: String,
}

/// Latest releases of an extension
#[derive(Deserialize)]
pub struct Latest {
    pub version: Version,
    pub assets: HashMap<String, Asset>,
}

/// A downloadable asset with checksum
#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub url: String,
    pub checksum: String,
}

/// All Infos about the current extension
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledExtensionInfo {
    pub manifest: ExtensionManifest,
    pub icon_path: PathBuf,
    pub enabled: bool,
}

/// Returns the extension info of all extensions
#[tauri::command]
pub fn get_installed_extensions(
    app_state: State<'_, AppState>,
) -> error::Result<Vec<InstalledExtensionInfo>> {
    let extensions_path = app_state.extensions_path.clone();
    let config = app_state.config.read()?.clone();
    let enabled = config.enabled;
    let ordered = config.ordered;

    let dirs = extensions_path.read_dir()?;

    let mut extensions = Vec::new();
    let paths: Vec<PathBuf> = dirs.filter_map(|e| Some(e.ok()?.path())).collect();

    for path in paths {
        if path.is_dir() {
            let manifest_path = extensions_path.join(&path).join("manifest.json");

            if manifest_path.exists() {
                let manifest_data = std::fs::read_to_string(&manifest_path)?;
                let manifest: ExtensionManifest = serde_json::from_str(&manifest_data)?;

                let icon_path = extensions_path.join(&path).join("icon.svg");

                extensions.push(InstalledExtensionInfo {
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
    }

    sort_by_order(&mut extensions, &ordered);

    Ok(extensions)
}

fn sort_by_order(v: &mut [InstalledExtensionInfo], ordered: &[String]) {
    let rank: HashMap<&str, usize> = ordered
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    v.sort_by_key(|i| {
        let r = rank
            .get(i.manifest.id.as_str())
            .cloned()
            .unwrap_or(usize::MAX);
        (r, i.manifest.id.clone())
    });
}

/// Fetches the online extension manifests.
#[tauri::command]
pub async fn fetch_online_extensions() -> error::Result<Vec<AvailableExtension>> {
    let res = reqwest::get(EXTENSIONS_URL).await?;
    let extensions: Vec<AvailableExtension> = res.json().await?;
    Ok(extensions)
}

/// Emits an extension update to the main window.
pub fn emit_extensions_update(app: &AppHandle) -> error::Result<()> {
    let app_state = app.state::<AppState>();
    let extensions = get_installed_extensions(app_state)?;

    if let Some(main_window) = app.get_webview_window("main") {
        main_window.emit("update-extensions", extensions)?;
    }

    Ok(())
}

/// Download extension latest of `latest_url`
pub async fn download_extension_latest(latest_url: &String) -> error::Result<Latest> {
    let resp = reqwest::get(latest_url).await?;
    let latest: Latest = resp.json().await?;

    Ok(latest)
}

/// Download extension assets
pub async fn download_extension(extension_latest: &Latest) -> error::Result<Vec<u8>> {
    let key = current_platform_key();

    if let Some(asset) = extension_latest.assets.get(&key) {
        // download the ZIP
        let resp = reqwest::get(&asset.url).await?;
        let bytes = resp.bytes().await?.to_vec();

        // verify SHA-256 checksum
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sum = hex::encode(hasher.finalize());
        if sum != asset.checksum {
            return Err(Error::Checksum);
        }

        Ok(bytes)
    } else {
        Err(Error::NoAssets)
    }
}

/// Returns the current platform
pub fn current_platform_key() -> String {
    let raw_os = std::env::consts::OS;
    let os = match raw_os {
        "macos" => "darwin",
        other_os => other_os,
    };
    let arch = std::env::consts::ARCH;
    format!("{}-{}", os, arch)
}

/// Install downloaded `bytes` to extensions folder by `extension_id`
pub async fn install_extension(
    id: &String,
    bytes: Vec<u8>,
    app_state: State<'_, AppState>,
) -> error::Result<()> {
    let extension_directory = app_state.extensions_path.join(id);
    let tmp = std::env::temp_dir().join(format!("{id}.tar.gz"));

    // empty extension directory if it exists
    if extension_directory.exists() {
        fs::remove_dir_all(&extension_directory)?;
    }

    // create the extension dir
    fs::create_dir_all(&extension_directory)?;

    // write tar.gz
    fs::write(&tmp, &bytes)?;

    // unzip
    let tar_gz = File::open(&tmp)?;
    let dec = GzDecoder::new(tar_gz);
    Archive::new(dec).unpack(extension_directory)?;

    // cleanup
    fs::remove_file(&tmp)?;
    Ok(())
}

/// Delete extension by `extension_id`
#[tauri::command]
pub async fn delete_extension(id: String, app: AppHandle) -> error::Result<()> {
    let app_state = app.state::<AppState>();

    let extension_directory = app_state.extensions_path.join(&id);
    if extension_directory.exists() {
        fs::remove_dir_all(&extension_directory)?;
    }

    // Remove from config
    {
        let mut config = app_state.config.write()?;

        config.enabled.retain(|f| f != &id);
        config.ordered.retain(|f| f != &id);

        // persist changes
        fs::write(&app_state.config_path, serde_json::to_string(&*config)?)?;
    }

    emit_extensions_update(&app)?;

    Ok(())
}

/// Downloads and installs an extension
#[tauri::command]
pub async fn download_and_install_extension(
    id: String,
    latest_url: String,
    app: AppHandle,
) -> error::Result<InstalledExtensionInfo> {
    let app_state = app.state::<AppState>();

    let latest = download_extension_latest(&latest_url).await?;
    let bytes = download_extension(&latest).await?;
    install_extension(&id, bytes, app_state.clone()).await?;

    // Emit update
    emit_extensions_update(&app)?;

    // Return newly installed extension
    let extensions_path = app_state.extensions_path.clone();
    let config = app_state.config.read()?.clone();
    let enabled = config.enabled;

    let manifest_path = extensions_path.join(&id).join("manifest.json");
    let manifest_data = std::fs::read_to_string(&manifest_path)?;
    let manifest: ExtensionManifest = serde_json::from_str(&manifest_data)?;

    let icon_path = extensions_path.join(&id).join("icon.svg");
    let this_enabled = enabled.contains(&id);

    Ok(InstalledExtensionInfo {
        manifest,
        icon_path,
        enabled: this_enabled,
    })
}
