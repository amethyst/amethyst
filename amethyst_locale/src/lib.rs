//! # amethyst_locale
//!
//! Localisation binding a `Fluent` file to an Asset<Locale> via the use of amethyst_assets.

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]

use amethyst_assets::{
    register_asset_type, register_importer, Asset, AssetProcessorSystem, AssetStorage, Format,
    LoadHandle, ProcessableAsset, ProcessingState,
};
use amethyst_error::Error;
pub use fluent::{concurrent::FluentBundle, FluentResource};
use serde::{Deserialize, Serialize};
use type_uuid::*;
use unic_langid::langid;

/// Internal representation of a Locale
#[derive(Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "442ea0e0-48d8-4a3c-ab36-faba55f2c0db"]
pub struct LocaleData(pub Vec<u8>);
register_asset_type!(LocaleData => Locale; AssetProcessorSystem<Locale>);

/// A loaded locale.
#[allow(missing_debug_implementations)]
#[derive(TypeUuid)]
#[uuid = "bf7713bb-6e1f-4873-bf0b-9d7c2253f46a"]
pub struct Locale {
    /// The bundle stores its resources for now.
    pub bundle: FluentBundle<FluentResource>,
}

impl Asset for Locale {
    fn name() -> &'static str {
        "locale::Locale"
    }
    type Data = LocaleData;
}

impl ProcessableAsset for Locale {
    fn process(
        data: LocaleData,
        _storage: &mut AssetStorage<Locale>,
        _handle: &LoadHandle,
    ) -> Result<amethyst_assets::ProcessingState<LocaleData, Locale>, Error> {
        let s = String::from_utf8(data.0)?;

        let resource = FluentResource::try_new(s).expect("Failed to parse locale data");
        let lang_en = langid!("en");
        let mut bundle = FluentBundle::new(vec![lang_en]);

        bundle
            .add_resource(resource)
            .expect("Failed to add resource");

        Ok(ProcessingState::Loaded(Locale { bundle }))
    }
}

/// Loads the strings from localisation files.
#[derive(Clone, Debug, Default, TypeUuid, Serialize, Deserialize)]
#[uuid = "fe7720ec-ecb5-4f59-8a09-656805eb4eff"]
pub struct FTLFormat;

register_importer!(".ftl", FTLFormat);
impl Format<LocaleData> for FTLFormat {
    fn name(&self) -> &'static str {
        "FTL"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<LocaleData, Error> {
        Ok(LocaleData(bytes))
    }
}
