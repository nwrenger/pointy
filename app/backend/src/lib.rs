use std::{
    ffi::CString,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    str::FromStr,
    sync::RwLock,
};

use libloading::{Library, Symbol};
use pointy_api::device_query::{DeviceQuery, DeviceState};
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItemBuilder, IsMenuItem, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, Wry,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_opener::OpenerExt;

struct AppState {
    config_path: PathBuf,
    extensions_path: PathBuf,
    app_config: RwLock<AppConfig>,
}

impl AppState {
    fn new(config_path: PathBuf, extensions_path: PathBuf, app_config: AppConfig) -> Self {
        Self {
            config_path,
            extensions_path,
            app_config: RwLock::new(app_config),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    autostart: bool,
    shortcut: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            shortcut: String::from("CommandOrControl+Shift+Space"),
        }
    }
}

#[derive(Deserialize)]
struct ExtensionManifest {
    name: String,
    description: String,
    icon: String,
}

#[derive(Serialize)]
struct ExtensionInfo {
    pub abbreveation: String,
    pub name: String,
    pub description: String,
    pub icon_path: String,
    pub enabled: bool,
}

/// Populates the tray menu and returns the currently activated extensions.
#[tauri::command]
fn update_extensions(
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<Vec<ExtensionInfo>, String> {
    let info = info_extensions(app_state)?;

    let mut extensions = vec![];

    for ExtensionInfo {
        abbreveation,
        name,
        enabled,
        ..
    } in &info
    {
        let check_item = CheckMenuItemBuilder::new(name)
            .id(format!("ext_{}", abbreveation))
            .checked(*enabled)
            .enabled(true)
            .build(&app)
            .map_err(|e| e.to_string())?;
        extensions.push(check_item);
    }

    let extensions_refs: Vec<&dyn IsMenuItem<Wry>> = extensions
        .iter()
        .map(|item| item as &dyn IsMenuItem<_>)
        .collect();

    let settings_i = SubmenuBuilder::new(&app, "Settings")
        .id("settings")
        .text("general", "General")
        .separator()
        .items(extensions_refs.as_slice())
        .separator()
        .text("manage_extensions", "Manage Extensions...")
        .text("download_extensions", "Download Extensions")
        .build()
        .map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(&app)
        .id("tray_menu")
        .about(None)
        .item(&settings_i)
        .separator()
        .quit()
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(tray) = app.tray_by_id("main_tray") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }

    Ok(info)
}

/// Should be called when the config changed.
#[tauri::command]
fn update_config(app: AppHandle, app_state: State<'_, AppState>) -> Result<(), String> {
    let old_app_config = app_state.app_config.read().map_err(|e| e.to_string())?;
    let app_config_data = fs::read_to_string(&app_state.config_path).map_err(|e| e.to_string())?;
    let new_app_config: AppConfig =
        serde_json::from_str(&app_config_data).map_err(|e| e.to_string())?;

    // Remove old shortcut
    let old_shortcut = Shortcut::from_str(&old_app_config.shortcut).map_err(|e| e.to_string())?;
    app.global_shortcut()
        .unregister(old_shortcut)
        .map_err(|e| e.to_string())?;

    drop(old_app_config);

    // Add new shortcut
    let new_shortcut = Shortcut::from_str(&new_app_config.shortcut).map_err(|e| e.to_string())?;
    app.global_shortcut()
        .register(new_shortcut)
        .map_err(|e| e.to_string())?;

    // Configure autostart
    let autostart_manager = app.autolaunch();
    if new_app_config.autostart {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }

    let mut w = app_state.app_config.write().map_err(|e| e.to_string())?;
    *w = new_app_config;

    Ok(())
}

/// Reads the "extensions/*" folder and returns a list of all extensions with their info.
#[tauri::command]
fn info_extensions(app_state: State<'_, AppState>) -> Result<Vec<ExtensionInfo>, String> {
    let extensions_path = app_state.extensions_path.clone();

    let enabled_path = extensions_path.join("enabled.json");

    let data = std::fs::read_to_string(&enabled_path)
        .map_err(|e| format!("Failed to read enabled.json: {}", e))?;

    let enabled: Vec<PathBuf> =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse enabled.json: {}", e))?;

    let dirs = extensions_path.read_dir().map_err(|e| e.to_string())?;

    let mut extensions = Vec::new();
    let mut paths: Vec<PathBuf> = dirs.filter_map(|e| Some(e.ok()?.path())).collect();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            let manifest_path = extensions_path.join(&path).join("manifest.json");

            let manifest_data = std::fs::read_to_string(&manifest_path)
                .map_err(|e| format!("Failed to read {}: {}", manifest_path.display(), e))?;

            let manifest: ExtensionManifest = serde_json::from_str(&manifest_data)
                .map_err(|e| format!("Failed to parse {}: {}", manifest_path.display(), e))?;

            let icon_path = extensions_path.join(&path).join(&manifest.icon);

            extensions.push(ExtensionInfo {
                abbreveation: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                name: manifest.name,
                description: manifest.description,
                icon_path: icon_path.to_string_lossy().to_string(),
                enabled: enabled.contains(&PathBuf::from(path.file_name().unwrap_or_default())),
            });
        }
    }

    Ok(extensions)
}

/// Toggles an `extension`
fn toggle_extension(extension: String, app_state: State<'_, AppState>) -> Result<(), String> {
    let enabled_path = app_state.extensions_path.join("enabled.json");
    let enabled_data = std::fs::read_to_string(&enabled_path)
        .map_err(|e| format!("Failed to read enabled.json: {}", e))?;
    let mut enabled: Vec<String> = serde_json::from_str(&enabled_data)
        .map_err(|e| format!("Failed to parse {}: {}", enabled_path.display(), e))?;

    if enabled.contains(&extension) {
        enabled.retain(|item| item != &extension);
    } else {
        enabled.push(extension);
    }

    let file = File::create(enabled_path).map_err(|e| e.to_string())?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &enabled).map_err(|e| e.to_string())?;

    Ok(())
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
    let lib_filename = "windows.dll";
    #[cfg(target_os = "macos")]
    let lib_filename = "mac.dylib";
    #[cfg(target_os = "linux")]
    let lib_filename = "linux.so";

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

        // As with the helper in `crate::api`, if the string is empty no error occurred
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
            let handle = app.handle();

            // Inital App Setup (Paths)
            let config_path = app.path().app_data_dir()?.join("config.json");
            if !config_path.exists() {
                fs::write(&config_path, serde_json::to_string(&AppConfig::default())?)?;
            }
            let extensions_path = app.path().app_data_dir()?.join("extensions");
            if !extensions_path.exists() {
                fs::create_dir_all(&extensions_path)?;
            }
            let enabled_path = extensions_path.join("enabled.json");
            if !enabled_path.exists() {
                fs::write(&enabled_path, "[]")?;
            }

            // Initial App Config
            let app_config_data = fs::read_to_string(&config_path)?;
            let app_config: AppConfig = serde_json::from_str(&app_config_data)?;

            // Global Shortcuts
            let main_window = handle.get_webview_window("main").unwrap();
            let scale_factor = main_window
                .current_monitor()?
                .map_or(1., |f| f.scale_factor());

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
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "general" => {
                        let general_config_path = app
                            .path()
                            .app_data_dir()
                            .unwrap_or_default()
                            .join("config.json");

                        if let Err(e) = app
                            .opener()
                            .open_path(general_config_path.to_string_lossy(), None::<&str>)
                        {
                            eprintln!("Error opening {}: {}", general_config_path.display(), e);
                        }
                    }
                    "manage_extensions" => {
                        let extensions_path = app
                            .path()
                            .app_data_dir()
                            .unwrap_or_default()
                            .join("extensions");

                        if let Err(e) = app
                            .opener()
                            .open_path(extensions_path.to_string_lossy(), None::<&str>)
                        {
                            eprintln!("Error opening {}: {}", extensions_path.display(), e);
                        }
                    }
                    "download_extensions" => {
                        // TODO correct url
                        let url = "https://github.com/nwrenger/pointy";
                        if let Err(e) = app.opener().open_url(url, None::<&str>) {
                            eprintln!("Error opening {}: {}", url, e);
                        }
                    }
                    id if id.starts_with("ext_") => {
                        let extension_key = id.trim_start_matches("ext_").to_string();
                        if let Err(e) =
                            toggle_extension(extension_key.clone(), app.state::<AppState>())
                        {
                            eprintln!("Error toggeling {}: {}", extension_key, e);
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Init Autostart, also from config
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ))?;

            let autostart_manager = app.autolaunch();

            if app_config.autostart {
                autostart_manager.enable()?;
            } else {
                autostart_manager.disable()?;
            }

            // Safe state
            app.manage(AppState::new(config_path, extensions_path, app_config));

            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            update_extensions,
            update_config,
            info_extensions,
            run_extension,
            read_to_string,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
