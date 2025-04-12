use pointy_lib::{
    api::{
        clipboard_write_image,
        device_query::{DeviceQuery, DeviceState},
    },
    extension_entry,
};
use xcap::Monitor;

extension_entry! {
    let device_state = DeviceState::new();
    let pos = device_state.get_mouse().coords;
    let monitor = Monitor::from_point(pos.0, pos.1).map_err(|e| e.to_string())?;
    let image_buffer = monitor.capture_image().map_err(|e| e.to_string())?;

    clipboard_write_image(image_buffer)
}
