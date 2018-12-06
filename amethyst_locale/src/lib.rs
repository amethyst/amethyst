//! # amethyst_locale
//!
//! Localisation binding a `Fluent` file to an Asset<Locale> via the use of amethyst_assets.
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]
use fluent::bundle::FluentBundle;

use amethyst_assets::{Asset, Handle, ProcessingState, Result, SimpleFormat};
use amethyst_core::specs::prelude::VecStorage;

/// Loads the strings from localisation files.
#[derive(Clone)]
pub struct LocaleFormat;

impl SimpleFormat<Locale> for LocaleFormat {
    const NAME: &'static str = "FTL";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<Locale> {
        let s = String::from_utf8(bytes)?;

        let mut bundle = FluentBundle::new::<&'static str>(&[]);
        bundle
            .add_messages(&s)
            .expect("Error creating fluent bundle!");
        Ok(Locale { bundle })
    }
}

impl Into<Result<ProcessingState<Locale>>> for Locale {
    fn into(self) -> Result<ProcessingState<Locale>> {
        Ok(ProcessingState::Loaded(self))
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
