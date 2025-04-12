use pointy_lib::{api::clipboard_write_text, define_plugin_command};
use rand::distr::{Alphanumeric, SampleString};

define_plugin_command! { |_app| {
        let mut rng = rand::rng();
        let text = Alphanumeric.sample_string(&mut rng, 12);

        clipboard_write_text(text)
    }
}
