use amethyst_assets::{Asset, Error, Handle, ResultExt, SimpleFormat};
use rusttype::{Font, FontCollection};
use specs::VecStorage;

/// A loaded set of fonts from a file.
pub struct FontAsset(pub Font<'static>);

/// A handle to font data stored with `amethyst_assets`.
pub type FontHandle = Handle<FontAsset>;

pub struct FontData(Font<'static>);

impl Asset for FontAsset {
    const NAME: &'static str = "ui::Font";
    type Data = FontData;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl Into<Result<FontAsset, Error>> for FontData {
    fn into(self) -> Result<FontAsset, Error> {
        Ok(FontAsset(self.0))
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
        FontCollection::from_bytes(bytes)
            .into_fonts()
            .nth(0)
            .map(|f| FontData(f))
            .chain_err(|| "Font parsing error")
    }
}
