use amethyst_assets::{Asset, BoxedErr, Handle, SimpleFormat};
use rusttype::{Font, FontCollection};
use specs::DenseVecStorage;

/// A loaded set of fonts from a file.
pub struct FontFileAsset(pub Vec<Font<'static>>);

pub struct FontData(Vec<Font<'static>>);

impl Asset for FontFileAsset {
    type Data = FontData;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl Into<Result<FontFileAsset, BoxedErr>> for FontData {
    fn into(self) -> Result<FontFileAsset, BoxedErr> {
        Ok(FontFileAsset(self.0))
    }
}

/// Loads font files, supports TrueType and **some** OpenType files.
///
/// OpenType is a superset of TrueType, so if your OpenType file uses any features that don't
/// exist in TrueType this will fail.
#[derive(Clone)]
pub struct FontFormat;

impl SimpleFormat<FontFileAsset> for FontFormat {
    const NAME: &'static str = "FONT";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<FontData, BoxedErr> {
        Ok(FontData(FontCollection::from_bytes(bytes).into_fonts().collect::<Vec<Font>>()))
    }
}
