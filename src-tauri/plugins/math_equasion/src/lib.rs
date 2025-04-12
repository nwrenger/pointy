use meval::eval_str;
use pointy_lib::{
    api::{clipboard_get_text, clipboard_write_text},
    define_plugin_command,
};

define_plugin_command! { |_app| {
        let clipboard_text = clipboard_get_text()?;
        let text = eval_str(clipboard_text)
            .map_err(|e| e.to_string())?
            .to_string();

        clipboard_write_text(text)
    }
}
