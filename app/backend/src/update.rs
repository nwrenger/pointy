use std::fs::{self, File};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::UpdaterExt;
use zip::ZipArchive;

use crate::{
    extensions::{emit_extensions_update, ExtensionLatest},
    get_extensions, AppState,
};

use tracing::{error, info, warn};

/// Updates the whole app
#[tauri::command]
pub async fn update_app(app: AppHandle) -> tauri_plugin_updater::Result<()> {
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
        app.restart();
    } else {
        info!("app is up-to-date");
    }

    Ok(())
}

/// Updates all extensions
#[tauri::command]
pub async fn update_extensions(app: AppHandle) -> Result<(), String> {
    let app_state = app.state::<AppState>();
    let extensions = get_extensions(app_state).map_err(|e| {
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
                        id = %extension.manifest.id,
                        "HTTP error fetching latest file"
                    );
                    return;
                }
                Err(e) => {
                    error!(id = %extension.manifest.id, %e, "request error fetching latest file");
                    return;
                }
            };
            let update: ExtensionLatest = match resp.json().await {
                Ok(u) => u,
                Err(e) => {
                    error!(
                        id = %extension.manifest.id,
                        %e,
                        "failed to parse latest file"
                    );
                    return;
                }
            };

            // check for version
            if update.version <= extension.manifest.version {
                info!(
                    id = %extension.manifest.id,
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
                        id = %extension.manifest.id,
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
                        id = %extension.manifest.id,
                        "HTTP error downloading asset"
                    );
                    return;
                }
                Err(e) => {
                    error!(id = %extension.manifest.id, %e, "error downloading asset");
                    return;
                }
            };
            let bytes = match resp2.bytes().await {
                Ok(b) => b.to_vec(),
                Err(e) => {
                    error!(id = %extension.manifest.id, %e, "error reading bytes");
                    return;
                }
            };

            // verify SHA-256 checksum
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let sum = hex::encode(hasher.finalize());
            if sum != asset.checksum {
                error!(
                    id = %extension.manifest.id,
                    expected = %asset.checksum,
                    actual = %sum,
                    "checksum mismatch"
                );
                return;
            }

            // install into a clean folder
            let ext_dir = state.extensions_path.join(&extension.manifest.id);
            let temp_zip = std::env::temp_dir().join(format!("{}.zip", extension.manifest.id));
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
                Ok(()) => info!(id = %extension.manifest.id, "installed extension update"),
                Err(e) => error!(id = %extension.manifest.id, %e, "failed to unpack extension"),
            }
        });

        handles.push(handle);
    }

    // Wait for all updating and return errors if any
    for h in handles {
        h.await.map_err(|e| e.to_string())?;
    }

    emit_extensions_update(&app).map_err(|e| e.to_string())?;

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
