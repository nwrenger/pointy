use meval::eval_str;
use pointy_api::{clipboard_get_text, clipboard_write_text, extension_entry};

extension_entry!(main);

fn main() -> Result<(), String> {
    let clipboard_text = clipboard_get_text()?;
    let text = eval_str(clipboard_text)
        .map_err(|e| e.to_string())?
        .to_string();

    clipboard_write_text(text)
}
