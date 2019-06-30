//! # amethyst_locale
//!
//! Localisation binding a `Fluent` file to an Asset<Locale> via the use of amethyst_assets.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

use amethyst_assets::{Asset, Format, Handle};
use amethyst_core::ecs::prelude::VecStorage;
use amethyst_error::Error;
use fluent::bundle::FluentBundle;
use serde::{Deserialize, Serialize};
use type_uuid::*;

/// Loads the strings from localisation files.
#[derive(Clone, Copy, Debug, Default, TypeUuid, Serialize, Deserialize)]
#[uuid = "fe7720ec-ecb5-4f59-8a09-656805eb4eff"]
pub struct LocaleFormat;

amethyst_assets::register_format_type!(Locale);

amethyst_assets::register_format!("FTL", LocaleFormat as Locale);
// Locale doesn't impl Serialize/Deserialize, so can't register importer :(
// amethyst_assets::register_importer!(".ftl", LocaleFormat);
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
#[derive(TypeUuid)]
#[uuid = "d9e1870c-9744-40b0-969d-15ec45ea7372"]
pub struct Locale {
    /// The message context.
    pub bundle: FluentBundle<'static>,
}

impl Asset for Locale {
    fn name() -> &'static str {
        "locale::Locale"
    }
    type Data = Locale;
    type HandleStorage = VecStorage<LocaleHandle>;
}
