use std::fs::{self, File};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::UpdaterExt;
use zip::ZipArchive;

use crate::{extensions::ExtensionLatest, update_system_tray, AppState};

use tracing::{error, info, warn};

/// Updates the whole app
pub async fn update_app(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
    let _ = update_system_tray(app, Some(true), None);
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    info!(
                        downloaded_bytes = downloaded,
                        total_bytes = ?content_length,
                        "download progress"
                    );
                },
                || {
                    info!("download finished");
                },
            )
            .await?;

        info!("update installed");
        let _ = update_system_tray(app, Some(false), None);
        app.restart();
    } else {
        let _ = update_system_tray(app, Some(false), None);
        info!("app is up-to-date");
    }

    Ok(())
}

/// Updates all extensions
pub fn update_extensions(app: &AppHandle) -> Result<(), String> {
    // show updating
    let extensions = update_system_tray(app, None, Some(true)).map_err(|e| {
        error!(%e, "failed to update system tray for extensions");
        e.to_string()
    })?;

    let mut handles = Vec::with_capacity(extensions.len());
    for extension in extensions {
        let app_handle = app.clone();

        let handle = tauri::async_runtime::spawn(async move {
            let state: State<'_, AppState> = app_handle.state();
            let key = current_platform_key();

            // fetch the remote update manifest
            let resp = match reqwest::get(&extension.manifest.latest_url).await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    error!(
                        status = %r.status(),
                        name = %extension.manifest.name,
                        "HTTP error fetching latest file"
                    );
                    return;
                }
                Err(e) => {
                    error!(name = %extension.manifest.name, %e, "request error fetching latest file");
                    return;
                }
            };
            let update: ExtensionLatest = match resp.json().await {
                Ok(u) => u,
                Err(e) => {
                    error!(
                        name = %extension.manifest.name,
                        %e,
                        "failed to parse latest file"
                    );
                    return;
                }
            };

            // check for version
            if update.version <= extension.manifest.version {
                info!(
                    name = %extension.manifest.name,
                    new = %update.version,
                    old = %extension.manifest.version,
                    "extension is up-to-date"
                );
                return;
            }

            // pick the right asset
            let asset = match update.assets.get(&key) {
                Some(a) => a.clone(),
                None => {
                    warn!(
                        name = %extension.manifest.name,
                        platform = %key,
                        "no asset for this platform"
                    );
                    return;
                }
            };

            // download the ZIP
            let resp2 = match reqwest::get(&asset.url).await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    error!(
                        status = %r.status(),
                        name = %extension.manifest.name,
                        "HTTP error downloading asset"
                    );
                    return;
                }
                Err(e) => {
                    error!(name = %extension.manifest.name, %e, "error downloading asset");
                    return;
                }
            };
            let bytes = match resp2.bytes().await {
                Ok(b) => b.to_vec(),
                Err(e) => {
                    error!(name = %extension.manifest.name, %e, "error reading bytes");
                    return;
                }
            };

            // verify SHA-256 checksum
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let sum = hex::encode(hasher.finalize());
            if sum != asset.checksum {
                error!(
                    name = %extension.manifest.name,
                    expected = %asset.checksum,
                    actual = %sum,
                    "checksum mismatch"
                );
                return;
            }

            // install into a clean folder
            let ext_dir = state.extensions_path.join(&extension.manifest.name);
            let temp_zip = std::env::temp_dir().join(format!("{}.zip", extension.manifest.name));
            let install_result = (|| -> std::io::Result<()> {
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
            })();

            match install_result {
                Ok(()) => info!(name = %extension.manifest.name, "installed extension update"),
                Err(e) => error!(name = %extension.manifest.name, %e, "failed to unpack extension"),
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
    let raw_os = std::env::consts::OS;
    let os = match raw_os {
        "macos" => "darwin",
        other_os => other_os,
    };
    let arch = std::env::consts::ARCH;
    format!("{}-{}", os, arch)
}
