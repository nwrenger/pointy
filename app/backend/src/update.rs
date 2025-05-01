use std::fs::{self, File};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::UpdaterExt;
use zip::ZipArchive;

use crate::{extensions::ExtensionUpdate, update_system_tray, AppState};

/// Updates the whole app
pub async fn update_app(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let _ = update_system_tray(app, Some(true), None);
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("Downloaded {downloaded} from {content_length:?}");
                },
                || {
                    println!("Download finished");
                },
            )
            .await?;

        println!("Update installed!");

        let _ = update_system_tray(app, Some(false), None);
        app.restart();
    }

    Ok(())
}

/// Updates all extensions
pub fn update_extensions(app: &AppHandle) -> Result<(), String> {
    // show updating
    let extensions = update_system_tray(app, None, Some(true)).map_err(|e| e.to_string())?;

    let mut handles = Vec::with_capacity(extensions.len());
    for extension in extensions {
        let app_handle = app.clone();

        let handle = tauri::async_runtime::spawn(async move {
            let state: State<'_, AppState> = app_handle.state();
            let key = current_platform_key();

            // fetch the remote update manifest
            let resp = match reqwest::get(&extension.manifest.manifest_url).await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    eprintln!(
                        "HTTP {} fetching manifest for {}",
                        r.status(),
                        &extension.manifest.name
                    );
                    return;
                }
                Err(e) => {
                    eprintln!("Request error for {}: {}", extension.manifest.name, e);
                    return;
                }
            };
            let update: ExtensionUpdate = match resp.json().await {
                Ok(u) => u,
                Err(e) => {
                    eprintln!(
                        "Failed to parse manifest for {}: {}",
                        extension.manifest.name, e
                    );
                    return;
                }
            };

            // check for version
            if update.version <= extension.manifest.version {
                println!(
                    "{} is up-to-date ({} ≤ {})",
                    extension.manifest.name, update.version, extension.manifest.version
                );
                return;
            }

            // pick the right asset
            let asset = match update.assets.get(&key) {
                Some(a) => a.clone(),
                None => {
                    eprintln!(
                        "No asset for {} on this platform {}",
                        extension.manifest.name, key
                    );
                    return;
                }
            };

            // download the ZIP
            let resp2 = match reqwest::get(&asset.url).await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    eprintln!(
                        "HTTP {} downloading asset for {}",
                        r.status(),
                        extension.manifest.name
                    );
                    return;
                }
                Err(e) => {
                    eprintln!(
                        "Error downloading asset for {}: {}",
                        extension.manifest.name, e
                    );
                    return;
                }
            };
            let bytes = match resp2.bytes().await {
                Ok(b) => b.to_vec(),
                Err(e) => {
                    eprintln!("Error reading bytes for {}: {}", extension.manifest.name, e);
                    return;
                }
            };

            // verify SHA-256 checksum
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let sum = hex::encode(hasher.finalize());
            if sum != asset.checksum {
                eprintln!("Checksum mismatch for {}", extension.manifest.name);
                return;
            }

            // install into a clean folder
            let ext_dir = state.extensions_path.join(&extension.manifest.name);
            let temp_zip = std::env::temp_dir().join(format!("{}.zip", extension.manifest.name));
            let install_result = || -> std::io::Result<()> {
                // wipe old
                if ext_dir.exists() {
                    fs::remove_dir_all(&ext_dir)?;
                }
                fs::create_dir_all(&ext_dir)?;

                // write ZIP
                fs::write(&temp_zip, &bytes)?;

                // unzip
                let f = File::open(&temp_zip)?;
                let mut archive = ZipArchive::new(f)?;
                archive.extract(&ext_dir)?;

                // cleanup
                fs::remove_file(&temp_zip)?;
                Ok(())
            }();

            if let Err(e) = install_result {
                eprintln!("Failed to unpack {}: {}", extension.manifest.name, e);
            } else {
                println!("Installed update to {}!", extension.manifest.name);
            }
        });

        handles.push(handle);
    }

    // clear updating
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        for h in handles {
            let _ = h.await;
        }
        let _ = update_system_tray(&app_handle, None, Some(false));
    });

    Ok(())
}

/// Returns the current platform
pub fn current_platform_key() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}
