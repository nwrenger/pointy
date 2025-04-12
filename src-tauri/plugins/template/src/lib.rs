use pointy_lib::{api::tauri::Manager, define_plugin_command};
use std::path::PathBuf;

define_plugin_command! { |app| {
        // Add now here your plugin code
        // Use `pointy_lib::api` for bundled dependencies and helper functions for the clipboard

        // And also access app state
        let plugins_path = app.state::<PathBuf>().inner();
        println!("{:?}", plugins_path);

        Ok(())
    }
}
