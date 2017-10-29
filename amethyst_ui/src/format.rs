use amethyst_assets::{Asset, Error, ErrorKind, Handle, SimpleFormat};
use rusttype::{Font, FontCollection};
use specs::DenseVecStorage;

/// A loaded set of fonts from a file.
pub struct FontAsset(pub Font<'static>);

/// A handle to font data stored with `amethyst_assets`
pub type FontHandle = Handle<FontAsset>;

pub struct FontData(Font<'static>);

impl Asset for FontAsset {
    type Data = FontData;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl Into<Result<FontAsset, Error>> for FontData {
    fn into(self) -> Result<FontAsset, Error> {
        Ok(FontAsset(self.0))
    }
}

/// Loads font files, supports TrueType and **some** OpenType files.
///
/// OpenType is a superset of TrueType, so if your OpenType file uses any features that don't
/// exist in TrueType this will fail.  This will only load the first font contained in a file.
/// If this is a problem for you please file an issue with Amethyst on GitHub.
#[derive(Clone)]
pub struct FontFormat;

impl SimpleFormat<FontAsset> for FontFormat {
    const NAME: &'static str = "FONT";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<FontData, Error> {
        FontCollection::from_bytes(bytes)
            .into_fonts()
            .nth(0)
            .map(|f| FontData(f))
            .ok_or(Error::from_kind(ErrorKind::Format("Font parsing error")))
    }
}
