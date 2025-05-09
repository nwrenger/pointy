use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::UpdaterExt;

use crate::{
    error,
    extensions::{
        download_extension, download_extension_latest, emit_extensions_update,
        get_installed_extensions, install_extension,
    },
    AppState,
};

use tracing::{error, info};

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
pub async fn update_extensions(app: AppHandle) -> error::Result<()> {
    let app_state = app.state::<AppState>();
    let extensions = get_installed_extensions(app_state)?;

    let mut handles = Vec::with_capacity(extensions.len());
    for extension in extensions {
        let app_handle = app.clone();

        let handle = tauri::async_runtime::spawn(async move {
            let state: State<'_, AppState> = app_handle.state();

            let latest = match download_extension_latest(&extension.manifest.latest_url).await {
                Ok(latest) => latest,
                Err(e) => {
                    error!(id = %extension.manifest.id, %e);
                    return;
                }
            };

            // check for version
            if latest.version <= extension.manifest.version {
                info!(
                    id = %extension.manifest.id,
                    new = %latest.version,
                    old = %extension.manifest.version,
                    "extension is up-to-date"
                );
                return;
            }

            let bytes = match download_extension(&latest).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(id = %extension.manifest.id, %e);
                    return;
                }
            };

            match install_extension(&extension.manifest.id, bytes, state).await {
                Ok(()) => info!(id = %extension.manifest.id, "installed extension update"),
                Err(e) => error!(id = %extension.manifest.id, %e, "failed to unpack extension"),
            }
        });

        handles.push(handle);
    }

    // Wait for all updating and return errors if any
    for h in handles {
        h.await?;
    }

    emit_extensions_update(&app)?;

    Ok(())
}
