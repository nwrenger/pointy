use std::str::FromStr;

use device_query::{DeviceQuery, DeviceState};
use image::{DynamicImage, Luma, RgbaImage};
use meval::eval_str;
use qrcode::QrCode;
use rand::distr::{Alphanumeric, SampleString};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use xcap::Monitor;

/// Captures a screenshot of the current monitor by mouse position and copies the result to the clipboard.
#[tauri::command]
async fn capture_screenshot(app: AppHandle) -> Result<(), String> {
    let monitor = find_current_monitor()?;
    let image_buffer = monitor.capture_image().map_err(|e| e.to_string())?;

    clipboard_write_image(image_buffer, &app)
}

pub fn find_current_monitor() -> Result<Monitor, String> {
    let device_state = DeviceState::new();
    let mouse_coords = device_state.get_mouse().coords;

    Monitor::from_point(mouse_coords.0, mouse_coords.1).map_err(|e| e.to_string())
}

/// Evaluates a math equasion and copies the result to the clipboard.
#[tauri::command]
fn evaluate_math_equasion(app: AppHandle) -> Result<(), String> {
    let clipboard_text = clipboard_get_text(&app)?;
    let text = eval_str(clipboard_text)
        .map_err(|e| e.to_string())?
        .to_string();

    clipboard_write_text(text, &app)
}

/// Generates a qrcode from copied text and saves it to downloads.
#[tauri::command]
fn generate_qrcode(app: AppHandle) -> Result<(), String> {
    let clipboard_text = clipboard_get_text(&app)?;

    let code = QrCode::new(clipboard_text.as_bytes()).map_err(|e| e.to_string())?;
    let image_luma = code.render::<Luma<u8>>().build();

    let dynamic_image = DynamicImage::ImageLuma8(image_luma);
    let image_buffer = dynamic_image.to_rgba8();

    clipboard_write_image(image_buffer, &app)
}

/// Creates a 12 character long very secure password and copies it to the clipboard.
#[tauri::command]
fn create_secure_password(app: AppHandle) -> Result<(), String> {
    let mut rng = rand::rng();
    let text = Alphanumeric.sample_string(&mut rng, 12);

    clipboard_write_text(text, &app)
}

pub fn clipboard_get_text(app: &AppHandle) -> Result<String, String> {
    app.clipboard().read_text().map_err(|e| e.to_string())
}

pub fn clipboard_write_text(text: String, app: &AppHandle) -> Result<(), String> {
    app.clipboard().write_text(&text).map_err(|e| e.to_string())
}

pub fn clipboard_write_image(image_buffer: RgbaImage, app: &AppHandle) -> Result<(), String> {
    let raw_pixels: &[u8] = image_buffer.as_raw();
    let width = image_buffer.width();
    let height = image_buffer.height();

    let image = Image::new(raw_pixels, width, height);

    app.clipboard()
        .write_image(&image)
        .map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            let main_window = handle.get_webview_window("main").unwrap();
            let scale_factor = main_window.current_monitor()?.unwrap().scale_factor();

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

            Ok(())
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            capture_screenshot,
            evaluate_math_equasion,
            generate_qrcode,
            create_secure_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
