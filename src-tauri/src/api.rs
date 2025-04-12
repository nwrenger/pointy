pub use arboard;
pub use device_query;
pub use image;
pub use tauri;

use std::borrow::Cow;

use arboard::{Clipboard, ImageData};
use image::RgbaImage;

/// This macro generates the FFI function `run_plugin_command` which:
/// - Converts the incoming raw pointer to a `&tauri::AppHandle`.
/// - Executes your custom logic (provided as a closure) that returns a Result<String, String>.
/// - Converts that result into a C string (`*mut c_char`) for return.
///
/// Usage:
/// ```rust
/// define_plugin_command! { |_app| {
///         // Your custom logic here. For example:
///         let clipboard_text = clipboard_get_text()?;   // You can call helper functions here.
///         let text = eval_str(clipboard_text)
///             .map_err(|e| e.to_string())?
///             .to_string();
///
///         clipboard_write_text(text)
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_plugin_command {
    ( |$app:ident| $body:block ) => {
        #[no_mangle]
        pub extern "C" fn run_plugin_command(
            app_handle_ptr: *const pointy_lib::api::tauri::AppHandle,
        ) -> *mut ::std::os::raw::c_char {
            let $app: &pointy_lib::api::tauri::AppHandle = unsafe {
                assert!(!app_handle_ptr.is_null(), "Received null pointer");
                &*app_handle_ptr
            };
            let result: Result<(), String> = (|| $body)();
            match result {
                Ok(_) => ::std::ffi::CString::new("").unwrap().into_raw(),
                Err(e) => ::std::ffi::CString::new(e).unwrap().into_raw(),
            }
        }
    };
}

/// Gets the text from the clipboard.
pub fn clipboard_get_text() -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

/// Gets an image from the clipboard.
pub fn clipboard_get_image() -> Result<ImageData<'static>, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_image().map_err(|e| e.to_string())
}

/// Writes text to the clipboard.
pub fn clipboard_write_text(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())
}

/// Writes text to the clipboard.
pub fn clipboard_write_html(text: String, alt_text: Option<String>) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_html(text, alt_text)
        .map_err(|e| e.to_string())
}

/// Writes an image to the clipboard.
pub fn clipboard_write_image(image_buffer: RgbaImage) -> Result<(), String> {
    let bytes: Cow<'_, [u8]> = Cow::Borrowed(image_buffer.as_raw());
    let width = image_buffer.width() as usize;
    let height = image_buffer.height() as usize;

    let image = ImageData {
        width,
        height,
        bytes,
    };

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_image(image).map_err(|e| e.to_string())
}

/// Clears the entire clipboard.
pub fn clipboard_clear() -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.clear().map_err(|e| e.to_string())
}
