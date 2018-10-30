use font_kit::handle::Handle as FontKitHandle;

use amethyst_assets::{AssetStorage, Loader, SimpleFormat};

use {
    font::systemfont::default_system_font,
    format::{FontAsset, FontHandle, TtfFormat},
};

/// Get the system default fonts.
/// If unable to, gets the local square.ttf font.
pub fn get_default_font(loader: &Loader, storage: &AssetStorage<FontAsset>) -> FontHandle {
    let system_font = default_system_font();

    match system_font {
        Ok(handle) => match handle {
            FontKitHandle::Path { .. } => unimplemented!(
                "Default system font was provided as a path, this is not yet supported.
				If you see this message, open an issue so that we know we need to implement it."
            ),
            FontKitHandle::Memory { bytes, .. } => {
                let font_data = TtfFormat.import(bytes.to_vec(), ());
                match font_data {
					Ok(data) => return loader.load_from_data(data, (), storage),
					Err(e) => warn!("Failed to load default system font from bytes. Falling back to built-in.\nError: {:?}", e),
				}
            }
        },
        Err(e) => warn!(
            "Failed to find suitable default system font. Falling back to built-in.\nError: {:?}",
            e
        ),
    }

    return loader.load_from_data(
        TtfFormat
            .import(include_bytes!("./square.ttf").to_vec(), ())
            .unwrap(),
        (),
        storage,
    );
}
