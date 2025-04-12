use pointy_lib::{api::clipboard_write_text, extension_entry};
use rand::distr::{Alphanumeric, SampleString};

extension_entry! {
    let mut rng = rand::rng();
    let text = Alphanumeric.sample_string(&mut rng, 12);

    clipboard_write_text(text)
}
