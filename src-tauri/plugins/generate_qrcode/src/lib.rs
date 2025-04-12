use pointy_lib::{
    api::{
        clipboard_get_text, clipboard_write_image,
        image::{DynamicImage, Luma},
    },
    define_plugin_command,
};
use qrcode::QrCode;

define_plugin_command! { |_app| {
        let clipboard_text = clipboard_get_text()?;

        let code = QrCode::new(clipboard_text.as_bytes()).map_err(|e| e.to_string())?;
        let image_luma = code.render::<Luma<u8>>().build();

        let dynamic_image = DynamicImage::ImageLuma8(image_luma);
        let image_buffer = dynamic_image.to_rgba8();

        clipboard_write_image(image_buffer)
    }
}
