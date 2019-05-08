//! # amethyst_locale
//!
//! Localisation binding a `Fluent` file to an Asset<Locale> via the use of amethyst_assets.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

use amethyst_assets::{Asset, Format, Handle};
use amethyst_core::ecs::prelude::VecStorage;
use amethyst_error::Error;
use fluent::bundle::FluentBundle;
use serde::{Deserialize, Serialize};

/// Loads the strings from localisation files.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LocaleFormat;

amethyst_assets::register_format_type!(Locale);

amethyst_assets::register_format!("FTL", LocaleFormat as Locale);
impl Format<Locale> for LocaleFormat {
    fn name(&self) -> &'static str {
        "FTL"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<Locale, Error> {
        let s = String::from_utf8(bytes)?;

        let mut bundle = FluentBundle::new::<&'static str>(&[]);
        bundle
            .add_messages(&s)
            .expect("Error creating fluent bundle!");
        Ok(Locale { bundle })
    }
}

/// A handle to a locale.
pub type LocaleHandle = Handle<Locale>;

/// A loaded locale.
pub struct Locale {
    /// The message context.
    pub bundle: FluentBundle<'static>,
}

impl Asset for Locale {
    const NAME: &'static str = "locale::Locale";
    type Data = Locale;
    type HandleStorage = VecStorage<LocaleHandle>;
}
