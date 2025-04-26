// Bundeling useful crates
pub use arboard;
pub use device_query;
pub use image;

use std::borrow::Cow;

use arboard::{Clipboard, ImageData};
use image::RgbaImage;

/// This macro generates the FFI function `run` which:
/// - Executes your custom logic provided as a function that returns a `Result<(), String>`.
/// - Converts that result into a `CString` (`*mut c_char`) for return.
/// ---
/// Usage:
/// ```rust
/// extension_entry!(main);
///
/// fn main() -> Result<(), String> {
///     let text = clipboard_get_text()?;
///     let words = text.split_whitespace().count();
///     clipboard_write_text(words.to_string())
/// }
/// ```
#[macro_export]
macro_rules! extension_entry {
    ($func:path) => {
        #[no_mangle]
        pub extern "C" fn run() -> *mut std::os::raw::c_char {
            let result: Result<(), String> = $func();
            match result {
                Ok(()) => std::ffi::CString::new("").unwrap().into_raw(),
                Err(e) => std::ffi::CString::new(e).unwrap().into_raw(),
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
