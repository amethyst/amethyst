use amethyst_assets::{
    register_asset_type, Asset, AssetProcessorSystem, AssetStorage, Format, LoadHandle,
    ProcessableAsset, ProcessingState,
};
use amethyst_error::{format_err, Error, ResultExt};
use glyph_brush::rusttype::Font;
use serde::{de, Deserialize, Serialize};
use type_uuid::TypeUuid;

/// A loaded set of fonts from a file.
#[derive(Clone, Debug, TypeUuid)]
#[uuid = "67bce379-48f7-4a35-bf54-243429a1816b"]
pub struct FontAsset(pub Font<'static>);

#[derive(Clone, Debug, TypeUuid)]
#[uuid = "85bac271-fe10-48da-85d2-151e93ce98d1"]
pub struct FontData(Font<'static>);

impl Serialize for FontData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("Font", "UNIMPLEMENTED")
    }
}

impl<'de> Deserialize<'de> for FontData {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Font::from_bytes(include_bytes!("./font/square.ttf").to_vec())
            .map(FontData)
            .map_err(|_e| de::Error::custom("impossible"))
    }
}

register_asset_type!(FontData => FontAsset; AssetProcessorSystem<FontAsset>);

impl Asset for FontAsset {
    fn name() -> &'static str {
        "ui::Font"
    }
    type Data = FontData;
}

impl ProcessableAsset for FontAsset {
    fn process(
        data: FontData,
        _storage: &mut AssetStorage<FontAsset>,
        _handle: &LoadHandle,
    ) -> Result<ProcessingState<FontData, FontAsset>, Error> {
        log::debug!("Loading Font");
        Ok(ProcessingState::Loaded(FontAsset(data.0)))
    }
}

/// Loads font files, supports TrueType and **some** OpenType files.
///
/// OpenType is a superset of TrueType, so if your OpenType file uses any features that don't
/// exist in TrueType this will fail.  This will only load the first font contained in a file.
/// If this is a problem for you please file an issue with Amethyst on GitHub.
#[derive(Clone, Debug, Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "2e974cc5-c0ad-4db5-8d43-40e7c69373d7"]
pub struct TtfFormat;

amethyst_assets::register_importer!(".ttf", TtfFormat);
impl Format<FontData> for TtfFormat {
    fn name(&self) -> &'static str {
        "TTF"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<FontData, Error> {
        Font::from_bytes(bytes)
            .map(FontData)
            .with_context(|_| format_err!("Font parsing error"))
    }
}
