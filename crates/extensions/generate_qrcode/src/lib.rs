use pointy_api::{
    clipboard_get_text, clipboard_write_image, extension_entry,
    image::{DynamicImage, Luma},
};
use qrcode::QrCode;

extension_entry!(main);

fn main() -> Result<(), String> {
    let clipboard_text = clipboard_get_text()?;

    let code = QrCode::new(clipboard_text.as_bytes()).map_err(|e| e.to_string())?;
    let image_luma = code.render::<Luma<u8>>().build();

    let dynamic_image = DynamicImage::ImageLuma8(image_luma);
    let image_buffer = dynamic_image.to_rgba8();

    clipboard_write_image(image_buffer)
}
