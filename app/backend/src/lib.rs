pub mod config;
pub mod error;
pub mod extensions;
pub mod update;

use std::{ffi::CString, fs, path::PathBuf, str::FromStr, sync::RwLock};

use config::{change_config, get_config, load_config, set_autolaunch, Config};
use error::Error;
use extensions::{
    delete_extension, download_and_install_extension, fetch_online_extensions,
    get_installed_extensions,
};
use libloading::{Library, Symbol};
use pointy_api::device_query::{DeviceQuery, DeviceState};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, LogicalPosition, LogicalSize, Manager, State,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tracing::info;
use update::{update_app, update_extensions};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AppState {
    pub config_path: PathBuf,
    pub extensions_path: PathBuf,
    pub config: RwLock<Config>,
}

impl AppState {
    pub fn new(config_path: PathBuf, extensions_path: PathBuf, config: Config) -> Self {
        Self {
            config_path,
            extensions_path,
            config: RwLock::new(config),
        }
    }
}

/// Gets app version.
#[tauri::command]
fn get_version() -> &'static str {
    PKG_VERSION
}

/// Runs a specified extension. It loads the appropriate dynamic library
/// for the current OS from the extension directory and calls the exported function.
/// The function is assumed to be:
///
///   pub extern "C" fn run() -> *mut c_char
///
#[tauri::command]
fn run_extension(extension_name: String, app_state: State<'_, AppState>) -> error::Result<()> {
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
        let lib = Library::new(&extensions_path)?;

        let func: Symbol<unsafe extern "C" fn() -> *mut std::os::raw::c_char> =
            lib.get(b"run\0")?;

        let raw_ptr = func();
        if raw_ptr.is_null() {
            return Err(Error::LibLoading(
                "Extension returned a null pointer".to_string(),
            ));
        }

        let cstring = CString::from_raw(raw_ptr);
        let result_str = cstring.into_string()?;

        // As with the helper in `pointy_api`, if the string is empty no error occurred
        if result_str.is_empty() {
            Ok(())
        } else {
            Err(Error::LibLoading(result_str))
        }
    }
}

/// Reads the file of a certain path to string.
#[tauri::command]
fn read_to_string(path: PathBuf) -> error::Result<String> {
    Ok(fs::read_to_string(path)?)
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
                fs::write(&config_path, serde_json::to_string(&Config::default())?)?;
            }
            let extensions_path = data_path.join("extensions");
            if !extensions_path.exists() {
                fs::create_dir_all(&extensions_path)?;
            }

            // Initial App Config
            let config = load_config(&config_path)?;

            // Main Window
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
                            .config
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
                .register(Shortcut::from_str(&config.shortcut)?)?;

            // System Tray
            let version = MenuItemBuilder::new(format!("{PKG_NAME} {PKG_VERSION}"))
                .enabled(false)
                .build(app)?;
            let settings = MenuItemBuilder::new("Settings")
                .id("settings")
                .accelerator("CmdOrCtrl+,")
                .build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&version)
                .separator()
                .item(&settings)
                .separator()
                .quit()
                .build()?;
            TrayIconBuilder::with_id("main_tray")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event({
                    move |app, event| {
                        if event.id.as_ref() == "settings" {
                            if let Some(settings_window) = app.get_webview_window("settings") {
                                settings_window.show().unwrap();
                                settings_window.set_focus().unwrap();
                                settings_window.emit("open-settings", ()).unwrap();
                            }
                        }
                    }
                })
                .build(app)?;

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Manage autolaunch
            handle.plugin(tauri_plugin_autostart::Builder::new().build())?;
            set_autolaunch(&config, &handle)?;

            // Save state
            app.manage(AppState::new(config_path, extensions_path, config));

            info!("application is setup");

            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_version,
            get_installed_extensions,
            fetch_online_extensions,
            run_extension,
            download_and_install_extension,
            delete_extension,
            update_app,
            update_extensions,
            get_config,
            change_config,
            read_to_string
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
