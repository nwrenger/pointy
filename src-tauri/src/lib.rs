pub mod api;

use std::{ffi::CString, fs, path::PathBuf, str::FromStr};

use device_query::{DeviceQuery, DeviceState};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

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
    // enabled - TODO
}

/// Reads the "extensions/enabled.json" file and returns a list of activated extensions.
#[tauri::command]
fn activated_extensions(app: AppHandle) -> Result<Vec<ExtensionInfo>, String> {
    let extensions_path = app.state::<PathBuf>().inner();

    let mut enabled_path = extensions_path.clone();
    enabled_path.push("enabled.json");

    let data = std::fs::read_to_string(&enabled_path)
        .map_err(|e| format!("Failed to read enabled.json: {}", e))?;

    let extension_folders: Vec<String> =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse enabled.json: {}", e))?;

    let mut extensions = Vec::new();
    for folder in extension_folders {
        let mut manifest_path = extensions_path.clone();
        manifest_path.push(&folder);
        manifest_path.push("manifest.json");

        let manifest_data = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read {}: {}", manifest_path.display(), e))?;

        let manifest: ExtensionManifest = serde_json::from_str(&manifest_data)
            .map_err(|e| format!("Failed to parse {}: {}", manifest_path.display(), e))?;

        let mut icon_path = extensions_path.clone();
        icon_path.push(&folder);
        icon_path.push(&manifest.icon);

        extensions.push(ExtensionInfo {
            abbreveation: folder,
            name: manifest.name,
            description: manifest.description,
            icon_path: icon_path.to_string_lossy().to_string(),
        });
    }

    Ok(extensions)
}

/// Runs a specified extension. It loads the appropriate dynamic library
/// for the current OS from the extension directory and calls the exported function.
/// The function is assumed to be:
///
///   pub extern "C" fn run() -> *mut c_char
///
#[tauri::command]
fn run_extension(app: AppHandle, extension_name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let lib_filename = "windows.dll";
    #[cfg(target_os = "macos")]
    let lib_filename = "mac.dylib";
    #[cfg(target_os = "linux")]
    let lib_filename = "linux.so";

    let mut extensions_path = app.state::<PathBuf>().inner().clone();
    extensions_path.push(&extension_name);
    extensions_path.push(lib_filename);

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

            let mut extensions_path = app.path().app_data_dir()?;
            extensions_path.push("extensions");
            if !extensions_path.exists() {
                fs::create_dir_all(&extensions_path).map_err(|_| tauri::Error::UnknownPath)?;
            }

            let mut enabled_path = extensions_path.clone();
            enabled_path.push("enabled.json");
            if !enabled_path.exists() {
                fs::write(&enabled_path, "[]").map_err(|_| tauri::Error::UnknownPath)?;
            }

            let main_window = handle.get_webview_window("main").unwrap();
            let scale_factor = main_window
                .current_monitor()?
                .map_or(1., |f| f.scale_factor());

            let new_window_shortcut = Shortcut::from_str("CommandOrControl+Shift+Space").unwrap();
            handle.plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &new_window_shortcut {
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

            app.global_shortcut().register(new_window_shortcut)?;

            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "settings" => {
                        println!("Settings was clicked!")
                    }
                    _ => {
                        unreachable!()
                    }
                })
                .build(app)?;

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // State
            app.manage(extensions_path);

            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            activated_extensions,
            run_extension,
            read_to_string
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
