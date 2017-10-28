use amethyst_assets::{Asset, BoxedErr, Handle, SimpleFormat};
use rusttype::{Font, FontCollection};
use specs::DenseVecStorage;

/// A loaded set of fonts from a file.
pub struct FontFileAsset(Vec<Font<'static>>);

impl Asset for FontFileAsset {
    type Data = Vec<Font<'static>>;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl From<Vec<Font<'static>>> for FontFileAsset {
    fn from(fonts: Vec<Font<'static>>) -> FontFileAsset {
        FontFileAsset(fonts)
    }
}

/// Loads font files, supports TrueType and **some** OpenType files.
///
/// OpenType is a superset of TrueType, so if your OpenType file uses any features that don't
/// exist in TrueType this will fail.
pub struct FontFormat;

impl SimpleFormat<FontFileAsset> for FontFormat {
    const NAME: &'static str = "FONT";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<Vec<Font<'static>>, BoxedErr> {
        Ok(FontCollection::from_bytes(bytes).into_fonts().collect::<Vec<Font>>())
    }
}
