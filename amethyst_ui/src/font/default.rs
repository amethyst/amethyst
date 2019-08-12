use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use font_kit::handle::Handle as FontKitHandle;
#[cfg(not(target_arch = "wasm32"))]
use super::systemfont::default_system_font;

use log::{error, warn};

use amethyst_assets::{AssetStorage, Format, Loader};

use crate::{
    format::{FontAsset, FontHandle, TtfFormat},
};

/// Get the system default fonts.
/// If unable to, gets the local square.ttf font.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_default_font(loader: &Loader, storage: &AssetStorage<FontAsset>) -> FontHandle {
    let system_font = default_system_font();

    match system_font {
        Ok(handle) => match handle {
            FontKitHandle::Path { path, .. } => {
                if let Some(file_extension) = path.extension() {
                    let format = match file_extension.to_str() {
                        Some(ext) => {
                            if ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf") {
                                Some(TtfFormat)
                            } else {
                                None
                            }
                        }
                        None => None,
                    };

                    if let Some(format) = format {
                        match fs::read(&path) {
                            Ok(bytes) => match format.import_simple(bytes) {
                                Ok(data) => return loader.load_from_data(data, (), storage),
                                Err(err) => warn!("System font at '{}' cannot be loaded. Fallback to default. Error: {}", path.display(), err),
                            },
                            Err(err) => warn!("System font at '{}' is not available for use. Fallback to default. Error: {}", path.display(), err)
                        }
                    } else {
                        error!("System font '{}' has unknown format", path.display());
                    }
                } else {
                    warn!("System font has no file extension!");
                }
            }
            FontKitHandle::Memory { bytes, .. } => {
                let font_data = TtfFormat.import_simple(bytes.to_vec());
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

    loader.load_from_data(
        TtfFormat
            .import_simple(include_bytes!("./square.ttf").to_vec())
            .expect("Unable to import fallback font './square.ttf'"),
        (),
        storage,
    )
}

#[cfg(target_arch = "wasm32")]
pub fn get_default_font(loader: &Loader, storage: &AssetStorage<FontAsset>) -> FontHandle {
    loader.load_from_data(
        TtfFormat
            .import_simple(include_bytes!("./square.ttf").to_vec())
            .expect("Unable to import fallback font './square.ttf'"),
        (),
        storage,
    )
}