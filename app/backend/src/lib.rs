pub mod config;
pub mod extensions;
pub mod update;

use std::{
    ffi::CString,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        RwLock,
    },
    time::Duration,
};

use config::{load_app_config, update_config, AppConfig};
use extensions::{info_extensions, toggle_extension, ExtensionInfo};
use libloading::{Library, Symbol};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use pointy_api::device_query::{DeviceQuery, DeviceState};
use tauri::{
    menu::{CheckMenuItemBuilder, IsMenuItem, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, Wry,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_opener::OpenerExt;
use tracing::{error, info};
use update::{update_app, update_extensions};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AppState {
    pub updating_app: AtomicBool,
    pub config_path: PathBuf,
    pub extensions_path: PathBuf,
    pub updating_extensions: AtomicBool,
    pub app_config: RwLock<AppConfig>,
    // Hold the watchers "alive"
    _extensions_watcher: RecommendedWatcher,
    _config_watcher: RecommendedWatcher,
}

impl AppState {
    pub fn new(
        config_path: PathBuf,
        extensions_path: PathBuf,
        app_config: AppConfig,
        _extensions_watcher: RecommendedWatcher,
        _config_watcher: RecommendedWatcher,
    ) -> Self {
        Self {
            updating_app: AtomicBool::new(false),
            config_path,
            extensions_path,
            updating_extensions: AtomicBool::new(false),
            app_config: RwLock::new(app_config),
            _extensions_watcher,
            _config_watcher,
        }
    }
}

/// Converts everything to a tauri error
pub fn to_tauri_error<E: std::fmt::Display>(e: E) -> tauri::Error {
    tauri::Error::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    ))
}

/// Generalized builder for the tray menu.
fn build_system_tray(
    app: &AppHandle,
    info: &Result<Vec<ExtensionInfo>, String>,
    updating_app: bool,
    updating_extensions: bool,
) -> tauri::Result<()> {
    let mut extensions = vec![];

    match info {
        Ok(info) => {
            for ExtensionInfo {
                manifest, enabled, ..
            } in info
            {
                let check_item = CheckMenuItemBuilder::new(&manifest.display_name)
                    .id(format!("ext_{}", &manifest.name))
                    .checked(*enabled)
                    .enabled(true)
                    .build(app)?;
                extensions.push(check_item);
            }

            if extensions.is_empty() {
                let empty_check_item = CheckMenuItemBuilder::new("No extensions found")
                    .id("empty_extensions")
                    .checked(false)
                    .enabled(false)
                    .build(app)?;
                extensions.push(empty_check_item);
            }
        }
        Err(e) => {
            let error_item = CheckMenuItemBuilder::new(format!("Error: {e}"))
                .id("error_extensions")
                .checked(false)
                .enabled(false)
                .build(app)?;
            extensions.push(error_item);
        }
    }

    let extensions_refs: Vec<&dyn IsMenuItem<Wry>> = extensions
        .iter()
        .map(|item| item as &dyn IsMenuItem<_>)
        .collect();

    let ext_update_label = if updating_extensions {
        "Updating all..."
    } else {
        "Check all for updates"
    };
    let updating_extensions = MenuItemBuilder::new(ext_update_label)
        .id("check_update_extensions")
        .enabled(!updating_extensions)
        .build(app)?;

    let extensions_i = SubmenuBuilder::new(app, "Extensions")
        .id("extensions")
        .items(extensions_refs.as_slice())
        .separator()
        .text("manage_extensions", "Manage")
        .item(&updating_extensions)
        .text("download_extensions", "Download")
        .build()?;

    let version = MenuItemBuilder::new(format!("{PKG_NAME} {PKG_VERSION}"))
        .enabled(false)
        .build(app)?;

    let update_label = if updating_app {
        "Updating..."
    } else {
        "Check for updates"
    };
    let updating_app_i = MenuItemBuilder::new(update_label)
        .id("check_update_app")
        .enabled(!updating_app)
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .id("tray_menu")
        .item(&version)
        .item(&updating_app_i)
        .separator()
        .text("config", "Config")
        .item(&extensions_i)
        .separator()
        .quit()
        .build()?;

    if let Some(tray) = app.tray_by_id("main_tray") {
        tray.set_menu(Some(menu))?;
    }

    Ok(())
}

/// Populates the tray menu and returns the used `Vec<ExtensionInfo>`.
fn update_system_tray(
    app: &AppHandle,
    updating: Option<bool>,
    updating_extensions: Option<bool>,
) -> tauri::Result<Vec<ExtensionInfo>> {
    let app_state = app.state::<AppState>();

    if let Some(updating) = updating {
        app_state.updating_app.store(updating, Ordering::Relaxed);
    }

    if let Some(updating_extensions) = updating_extensions {
        app_state
            .updating_extensions
            .store(updating_extensions, Ordering::Relaxed);
    }

    let updating_app = app_state.updating_app.load(Ordering::Relaxed);
    let updating_extensions = app_state.updating_extensions.load(Ordering::Relaxed);
    let info = info_extensions(app_state).map_err(|e| e.to_string());

    build_system_tray(app, &info, updating_app, updating_extensions)?;

    Ok(info.unwrap_or_default())
}

/// Get the inital extensions, please run once. After that use the event `update-extensions`.
#[tauri::command]
fn initial_extensions(app: AppHandle) -> Result<Vec<ExtensionInfo>, String> {
    update_system_tray(&app, None, None).map_err(|e| e.to_string())
}

/// Runs a specified extension. It loads the appropriate dynamic library
/// for the current OS from the extension directory and calls the exported function.
/// The function is assumed to be:
///
///   pub extern "C" fn run() -> *mut c_char
///
#[tauri::command]
fn run_extension(extension_name: String, app_state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let lib_filename = "lib.dll";
    #[cfg(target_os = "macos")]
    let lib_filename = "lib.dylib";
    #[cfg(target_os = "linux")]
    let lib_filename = "lib.so";

    let extensions_path = app_state
        .extensions_path
        .join(&extension_name)
        .join(lib_filename);

    unsafe {
        let lib = Library::new(&extensions_path).map_err(|e| {
            format!(
                "Failed to load extension {}: {}",
                extensions_path.display(),
                e
            )
        })?;

        let func: Symbol<unsafe extern "C" fn() -> *mut std::os::raw::c_char> =
            lib.get(b"run\0")
                .map_err(|e| format!("Failed to find symbol 'run': {}", e))?;

        let raw_ptr = func();
        if raw_ptr.is_null() {
            return Err("Extension returned a null pointer".to_string());
        }

        let cstring = CString::from_raw(raw_ptr);
        let result_str = cstring
            .into_string()
            .map_err(|e| format!("Failed to convert C string: {}", e))?;

        // As with the helper in `pointy_api`, if the string is empty no error occurred
        if result_str.is_empty() {
            Ok(())
        } else {
            Err(result_str)
        }
    }
}

/// Reads the file of a certain path to string.
#[tauri::command]
fn read_to_string(path: PathBuf) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Starting point for desktop app
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            info!("application setup starting");

            let handle = app.handle().clone();

            // Inital App Setup (Paths)
            let data_path = app.path().app_data_dir()?;
            if !data_path.exists() {
                fs::create_dir_all(&data_path)?;
            }
            let config_path = data_path.join("config.json");
            if !config_path.exists() {
                fs::write(&config_path, serde_json::to_string(&AppConfig::default())?)?;
            }
            let extensions_path = data_path.join("extensions");
            if !extensions_path.exists() {
                fs::create_dir_all(&extensions_path)?;
            }
            let enabled_path = extensions_path.join("enabled.json");
            if !enabled_path.exists() {
                fs::write(&enabled_path, "[]")?;
            }

            // Initial App Config
            let app_config = load_app_config(&config_path)?;

            // Window Size
            let main_window = handle.get_webview_window("main").unwrap();
            let scale_factor = main_window
                .current_monitor()?
                .map_or(1., |f| f.scale_factor());

            // Global Shortcuts
            handle.plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        let current_window_shortcut_str = app
                            .state::<AppState>()
                            .app_config
                            .read()
                            .unwrap()
                            .shortcut
                            .clone();
                        let current_window_shortcut =
                            Shortcut::from_str(&current_window_shortcut_str).unwrap();

                        if shortcut == &current_window_shortcut {
                            match event.state() {
                                ShortcutState::Pressed => {
                                    // get mouse position
                                    let device_state = DeviceState::new();
                                    let pos = device_state.get_mouse().coords;

                                    // get window size
                                    let size: LogicalSize<u32> = LogicalSize::from_physical(
                                        main_window.outer_size().unwrap(),
                                        scale_factor,
                                    );

                                    // window size divided by 2 for centering relative to the mouse position
                                    let logical_pos = LogicalPosition::new(
                                        pos.0.saturating_sub((size.width / 2) as i32),
                                        pos.1.saturating_sub((size.height / 2) as i32),
                                    );

                                    main_window.set_position(logical_pos).unwrap();
                                    main_window.show().unwrap();
                                    main_window.set_focus().unwrap();
                                }
                                ShortcutState::Released => {
                                    main_window.hide().unwrap();
                                    main_window.emit("select-option", ()).unwrap();
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            // Shortcut from Config
            app.global_shortcut()
                .register(Shortcut::from_str(&app_config.shortcut)?)?;

            // Init Tray
            let item = MenuItemBuilder::new("Loading...")
                .enabled(false)
                .build(app)?;
            let menu = MenuBuilder::new(app).item(&item).build()?;
            TrayIconBuilder::with_id("main_tray")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event({
                    let config_path_copy = config_path.clone();
                    let extensions_path_copy = extensions_path.clone();
                    move |app, event| match event.id.as_ref() {
                        "check_update_app" => {
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = update_app(&app_clone).await {
                                    error!("on updating: {}", e);
                                }
                            });
                        }
                        "config" => {
                            if let Err(e) = app
                                .opener()
                                .open_path(config_path_copy.to_string_lossy(), None::<&str>)
                            {
                                error!("on opening {}: {}", config_path_copy.display(), e);
                            }
                        }
                        "manage_extensions" => {
                            if let Err(e) = app
                                .opener()
                                .open_path(extensions_path_copy.to_string_lossy(), None::<&str>)
                            {
                                error!("on opening {}: {}", extensions_path_copy.display(), e);
                            }
                        }
                        "check_update_extensions" => {
                            if let Err(e) = update_extensions(app) {
                                error!("on updating extensions: {}", e);
                            }
                        }
                        "download_extensions" => {
                            // TODO: correct url
                            let url = "https://github.com/nwrenger/pointy";
                            if let Err(e) = app.opener().open_url(url, None::<&str>) {
                                error!("on opening {}: {}", url, e);
                            }
                        }
                        id if id.starts_with("ext_") => {
                            let extension_key = id.trim_start_matches("ext_").to_string();
                            if let Err(e) =
                                toggle_extension(extension_key.clone(), app.state::<AppState>())
                            {
                                error!("on toggeling {}: {}", extension_key, e);
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Init Autostart, also from config
            handle.plugin(tauri_plugin_autostart::Builder::new().build())?;
            let autostart_manager = app.autolaunch();
            if app_config.autostart {
                if !autostart_manager.is_enabled()? {
                    autostart_manager.enable()?;
                }
            } else if autostart_manager.is_enabled()? {
                autostart_manager.disable()?;
            }

            let watcher_config = Config::default()
                .with_poll_interval(Duration::from_millis(100))
                .with_compare_contents(true);

            // Watch conifg file
            let mut config_watcher = RecommendedWatcher::new(
                {
                    let handle_copy = handle.clone();
                    move |e| {
                        #[allow(clippy::redundant_pattern_matching)]
                        if let Ok(_) = e {
                            if let Err(e) = update_config(&handle_copy) {
                                error!("on watching config: {}", e);
                            }
                        }
                    }
                },
                watcher_config,
            )?;
            config_watcher.watch(&config_path, RecursiveMode::Recursive)?;

            // Watch extensions directory
            let mut extensions_watcher = RecommendedWatcher::new(
                move |e| {
                    #[allow(clippy::redundant_pattern_matching)]
                    if let Ok(_) = e {
                        match update_system_tray(&handle, None, None) {
                            Ok(extensions) => {
                                let main_window = handle.get_webview_window("main").unwrap();
                                main_window.emit("update-extensions", extensions).unwrap();
                            }
                            Err(e) => {
                                error!("on watching extensions: {}", e);
                            }
                        }
                    }
                },
                watcher_config,
            )?;
            extensions_watcher.watch(&extensions_path, RecursiveMode::Recursive)?;

            // Safe state
            app.manage(AppState::new(
                config_path,
                extensions_path,
                app_config,
                extensions_watcher,
                config_watcher,
            ));

            info!("application is setup");

            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            initial_extensions,
            run_extension,
            read_to_string
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
