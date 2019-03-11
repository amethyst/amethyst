use crate::types::Texture;
use amethyst_assets::SimpleFormat;
use amethyst_error::Error;
use rendy::texture::{
    image::{load_from_image, ImageTextureConfig},
    TextureBuilder,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ImageFormat;

impl SimpleFormat<Texture> for ImageFormat {
    const NAME: &'static str = "IMAGE";
    type Options = ImageTextureConfig;

    fn import(
        &self,
        bytes: Vec<u8>,
        options: ImageTextureConfig,
    ) -> Result<TextureBuilder<'static>, Error> {
        load_from_image(&bytes, options).map_err(|e| e.compat().into())
    }
}
