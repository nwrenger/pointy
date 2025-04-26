use pointy_api::{clipboard_write_text, extension_entry};
use rand::distr::{Alphanumeric, SampleString};

extension_entry!(main);

fn main() -> Result<(), String> {
    let mut rng = rand::rng();
    let text = Alphanumeric.sample_string(&mut rng, 12);

    clipboard_write_text(text)
}
