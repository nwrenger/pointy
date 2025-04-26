use pointy_api::{clipboard_get_text, clipboard_write_text, extension_entry};

extension_entry!(main);

fn main() -> Result<(), String> {
    let text = clipboard_get_text()?;

    let words = text.split_whitespace().count();
    let characters = text.chars().count();
    let lines = text.lines().count();

    clipboard_write_text(format!(
        r#"Words: {words}
Characters (no spaces): {characters}
Lines: {lines}
"#
    ))
}
