use gfx_glyph::Font;
use serde::{Deserialize, Serialize};

use amethyst_assets::{Asset, Error, Handle, ProcessingState, ResultExt, SimpleFormat};
use amethyst_core::specs::prelude::VecStorage;

/// A loaded set of fonts from a file.
#[derive(Clone)]
pub struct FontAsset(pub Font<'static>);

/// A handle to font data stored with `amethyst_assets`.
pub type FontHandle = Handle<FontAsset>;

#[derive(Clone)]
pub struct FontData(Font<'static>);

impl Asset for FontAsset {
    const NAME: &'static str = "ui::Font";
    type Data = FontData;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl Into<Result<ProcessingState<FontAsset>, Error>> for FontData {
    fn into(self) -> Result<ProcessingState<FontAsset>, Error> {
        Ok(ProcessingState::Loaded(FontAsset(self.0)))
    }
}

/// Identical to TtfFormat.
///
/// Loads font files, supports TrueType and **some** OpenType files.
///
/// OpenType is a superset of TrueType, so if your OpenType file uses any features that don't
/// exist in TrueType this will fail.  This will only load the first font contained in a file.
/// If this is a problem for you please file an issue with Amethyst on GitHub.
pub type OtfFormat = TtfFormat;

/// Loads font files, supports TrueType and **some** OpenType files.
///
/// OpenType is a superset of TrueType, so if your OpenType file uses any features that don't
/// exist in TrueType this will fail.  This will only load the first font contained in a file.
/// If this is a problem for you please file an issue with Amethyst on GitHub.
#[derive(Clone)]
pub struct TtfFormat;

impl SimpleFormat<FontAsset> for TtfFormat {
    const NAME: &'static str = "TTF/OTF";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<FontData, Error> {
        Font::from_bytes(bytes)
            .map(FontData)
            .chain_err(|| "Font parsing error")
    }
}

/// Wrapper format for all core supported Font formats
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FontFormat {
    /// TTF Format
    Ttf,
    /// OTF Format
    Otf,
}

impl SimpleFormat<FontAsset> for FontFormat {
    const NAME: &'static str = "FontFormat";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<FontData, Error> {
        match *self {
            FontFormat::Ttf | FontFormat::Otf => TtfFormat.import(bytes, ()),
        }
    }
}
