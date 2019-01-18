use std::result::Result as StdResult;

use error_chain::bail;
use gfx::{
    format::{ChannelType, SurfaceType, SurfaceTyped},
    texture::SamplerInfo,
    traits::Pod,
};
use image::{DynamicImage, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};

use amethyst_assets::{
    AssetStorage, Format, Handle, Loader, PrefabData, PrefabError, ProcessingState,
    ProgressCounter, Result, ResultExt, SimpleFormat,
};
use amethyst_core::specs::prelude::{Entity, Read, ReadExpect};

use crate::{
    tex::{FilterMethod, Texture, TextureBuilder},
    types::SurfaceFormat,
    Renderer,
};

/// Additional texture metadata that can be passed to the asset loader or added to the prefab.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextureMetadata {
    /// The sampler info describes how to read from the texture, thus specifies
    /// filter and wrap mode.
    /// The default is nearest filtering (`FilterMethod::Scale`) and clamping (`WrapMode::Clamp`).
    #[serde(default = "serde_helper::default_sampler")]
    pub sampler: SamplerInfo,
    /// Mipmapping levels. The default is one level.
    #[serde(default = "serde_helper::default_mip_levels")]
    pub mip_levels: u8,
    /// Dynamic texture
    #[serde(default)]
    pub dynamic: bool,
    /// The surface type of the texture which describes the number of color channels and their size.
    /// In simpler words, this defines the color format, e.g. RGBA 32-bit.
    ///
    /// The default is `R8_G8_B8_A8`.
    #[serde(default = "SurfaceFormat::get_surface_type")]
    pub format: SurfaceType,
    /// The dimensions of the texture. Only needed for raw image data (`TextureData::U8` etc).
    #[serde(default)]
    pub size: Option<(u16, u16)>,
    /// The channel type which describes the data format of the channels (e.g. how the red value
    /// is stored).
    ///
    /// This is usually `Srgb` for color textures, normalmaps & similar mostly use `Unorm`
    /// (which represents a value between `0.0` and `1.0`).
    pub channel: ChannelType,
}

impl TextureMetadata {
    /// Creates texture metadata with `Unorm` channel type. This is used for normal / displacement
    /// maps. For color textures you most likely want to use `TextureMetadata::srgb`.
    pub fn unorm() -> Self {
        TextureMetadata {
            sampler: serde_helper::default_sampler(),
            mip_levels: serde_helper::default_mip_levels(),
            dynamic: false,
            format: SurfaceFormat::get_surface_type(),
            size: None,
            channel: ChannelType::Unorm,
        }
    }

    /// Creates texture metadata for `Srgb` textures. This is usually the case for color textures.
    ///
    /// For the values of all the other fields please refer to the documentation of the respective
    /// field.
    pub fn srgb() -> Self {
        TextureMetadata {
            channel: ChannelType::Srgb,
            ..TextureMetadata::unorm()
        }
    }

    /// Creates texture metadata for `Srgb` texture but without texture filtering. This is usually
    /// the case for color textures for 2D sprites and pixel art textures.
    ///
    /// For the values of all the other fields please refer to the documentation of the respective
    /// field.
    ///
    /// Wrap mode is set to `WrapMode::Clamp` by default.
    pub fn srgb_scale() -> Self {
        TextureMetadata::srgb().with_filter(FilterMethod::Scale)
    }

    /// Sampler info
    pub fn with_sampler(mut self, info: SamplerInfo) -> Self {
        self.sampler = info;
        self
    }

    /// Sets the filter method of the sampler.
    pub fn with_filter(mut self, filter: FilterMethod) -> Self {
        self.sampler.filter = filter;

        self
    }

    /// Mipmapping
    pub fn with_mip_levels(mut self, mip_levels: u8) -> Self {
        self.mip_levels = mip_levels;
        self
    }

    /// Surface type
    pub fn with_format(mut self, format: SurfaceType) -> Self {
        self.format = format;
        self
    }

    /// Channel type
    pub fn with_channel(mut self, channel: ChannelType) -> Self {
        self.channel = channel;
        self
    }

    /// Texture size
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.size = Some((width, height));

        self
    }

    /// Texture is dynamic
    pub fn dynamic(mut self, d: bool) -> Self {
        self.dynamic = d;
        self
    }
}

/// Texture data for loading
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TextureData {
    /// Image data
    #[serde(skip)]
    Image(ImageData, TextureMetadata),

    /// Color
    Rgba([f32; 4], TextureMetadata),

    /// Float data
    F32(Vec<f32>, TextureMetadata),

    /// Float data
    F64(Vec<f64>, TextureMetadata),

    /// Byte data
    U8(Vec<u8>, TextureMetadata),

    /// Byte data
    U16(Vec<u16>, TextureMetadata),

    /// Byte data
    U32(Vec<u32>, TextureMetadata),

    /// Byte data
    U64(Vec<u64>, TextureMetadata),
}

impl From<[f32; 4]> for TextureData {
    fn from(color: [f32; 4]) -> Self {
        TextureData::Rgba(color, TextureMetadata::srgb())
    }
}

impl From<[f32; 3]> for TextureData {
    fn from(color: [f32; 3]) -> Self {
        [color[0], color[1], color[2], 1.0].into()
    }
}

impl TextureData {
    /// Creates texture data from color.
    pub fn color(value: [f32; 4]) -> Self {
        TextureData::Rgba(value, TextureMetadata::srgb())
    }
}

impl<'a> PrefabData<'a> for TextureData {
    type SystemData = (ReadExpect<'a, Loader>, Read<'a, AssetStorage<Texture>>);
    type Result = Handle<Texture>;

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> StdResult<Handle<Texture>, PrefabError> {
        Ok(system_data
            .0
            .load_from_data(self.clone(), (), &system_data.1))
    }
}

/// `PrefabData` for loading `Texture`s.
///
/// Will not add any `Component`s to the `Entity`, will only return a `Handle`
///
/// ### Type parameters:
///
/// - `F`: `Format` to use for loading the `Texture`s from file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TexturePrefab<F>
where
    F: Format<Texture, Options = TextureMetadata>,
{
    /// Texture data
    Data(TextureData),

    /// Load file with format
    File(String, F, TextureMetadata),

    /// Clone handle only
    #[serde(skip)]
    Handle(Handle<Texture>),
}

impl<'a, F> PrefabData<'a> for TexturePrefab<F>
where
    F: Format<Texture, Options = TextureMetadata> + Clone + Sync,
{
    type SystemData = (ReadExpect<'a, Loader>, Read<'a, AssetStorage<Texture>>);

    type Result = Handle<Texture>;

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> StdResult<Handle<Texture>, PrefabError> {
        let handle = match *self {
            TexturePrefab::Data(ref data) => {
                system_data
                    .0
                    .load_from_data(data.clone(), (), &system_data.1)
            }

            TexturePrefab::File(..) => unreachable!(),

            TexturePrefab::Handle(ref handle) => handle.clone(),
        };
        Ok(handle)
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> StdResult<bool, PrefabError> {
        let handle = match *self {
            TexturePrefab::Data(ref data) => Some(system_data.0.load_from_data(
                data.clone(),
                progress,
                &system_data.1,
            )),

            TexturePrefab::File(ref name, ref format, ref options) => Some(system_data.0.load(
                name.as_ref(),
                format.clone(),
                options.clone(),
                progress,
                &system_data.1,
            )),

            TexturePrefab::Handle(_) => None,
        };
        if let Some(handle) = handle {
            *self = TexturePrefab::Handle(handle);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    /// The raw image data.
    pub rgba: RgbaImage,
}

fn load_into_rgba8_from_memory(
    data: &[u8],
    options: TextureMetadata,
    format: ImageFormat,
) -> Result<TextureData> {
    use image::load_from_memory_with_format;
    load_from_memory_with_format(data, format)
        .map(|image| {
            match image {
                DynamicImage::ImageRgba8(im) => im,
                _ => {
                    // TODO: Log performance warning.
                    image.to_rgba()
                }
            }
        })
        .map(|rgba| TextureData::Image(ImageData { rgba }, options))
        // TODO: Add more context? File path or containing gltf archive?
        .chain_err(|| "Image decoding failed")
}

/// Allows loading of jpg or jpeg files.
#[derive(Clone, Deserialize, Serialize)]
pub struct JpgFormat;

impl JpgFormat {
    /// Load Jpg from memory buffer
    pub fn from_data(data: &[u8], options: TextureMetadata) -> Result<TextureData> {
        load_into_rgba8_from_memory(data, options, ImageFormat::JPEG)
    }
}

impl SimpleFormat<Texture> for JpgFormat {
    const NAME: &'static str = "JPEG";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        JpgFormat::from_data(&bytes, options)
    }
}

/// Allows loading of PNG files.
#[derive(Clone, Deserialize, Serialize)]
pub struct PngFormat;

impl PngFormat {
    /// Load Png from memory buffer
    pub fn from_data(data: &[u8], options: TextureMetadata) -> Result<TextureData> {
        load_into_rgba8_from_memory(data, options, ImageFormat::PNG)
    }
}

impl SimpleFormat<Texture> for PngFormat {
    const NAME: &'static str = "PNG";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        PngFormat::from_data(&bytes, options)
    }
}

/// Allows loading of BMP files.
#[derive(Clone, Deserialize, Serialize)]
pub struct BmpFormat;

impl SimpleFormat<Texture> for BmpFormat {
    const NAME: &'static str = "BMP";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        // TODO: consider reading directly into GPU-visible memory
        // TODO: as noted by @omni-viral.
        load_into_rgba8_from_memory(&bytes, options, ImageFormat::BMP)
    }
}

/// Allows loading of TGA files.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TgaFormat;

impl TgaFormat {
    /// Loads a TGA image from a byte slice.
    pub fn from_data(data: &[u8], options: TextureMetadata) -> Result<TextureData> {
        load_into_rgba8_from_memory(data, options, ImageFormat::TGA)
    }
}

impl SimpleFormat<Texture> for TgaFormat {
    const NAME: &'static str = "TGA";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        TgaFormat::from_data(&bytes, options)
    }
}

/// Create a texture asset.
pub fn create_texture_asset(
    data: TextureData,
    renderer: &mut Renderer,
) -> Result<ProcessingState<Texture>> {
    use self::TextureData::*;
    let t = match data {
        Image(image_data, options) => {
            create_texture_asset_from_image(image_data, options, renderer)
        }

        Rgba(color, options) => {
            let tb = apply_options(Texture::from_color_val(color), options);
            renderer
                .create_texture(tb)
                .chain_err(|| "Failed to build texture")
        }

        F32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer
                .create_texture(tb)
                .chain_err(|| "Failed to build texture")
        }

        F64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer
                .create_texture(tb)
                .chain_err(|| "Failed to build texture")
        }

        U8(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer
                .create_texture(tb)
                .chain_err(|| "Failed to build texture")
        }

        U16(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer
                .create_texture(tb)
                .chain_err(|| "Failed to build texture")
        }

        U32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer
                .create_texture(tb)
                .chain_err(|| "Failed to build texture")
        }

        U64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer
                .create_texture(tb)
                .chain_err(|| "Failed to build texture")
        }
    };
    t.map(ProcessingState::Loaded)
}

fn apply_options<D, T>(tb: TextureBuilder<D, T>, metadata: TextureMetadata) -> TextureBuilder<D, T>
where
    D: AsRef<[T]>,
    T: Pod + Copy,
{
    let builder = tb
        .with_sampler(metadata.sampler)
        .mip_levels(metadata.mip_levels)
        .dynamic(metadata.dynamic)
        .with_format(metadata.format)
        .with_channel_type(metadata.channel);
    if let Some((x, y)) = metadata.size {
        builder.with_size(x, y)
    } else {
        builder
    }
}

fn create_texture_asset_from_image(
    image: ImageData,
    options: TextureMetadata,
    renderer: &mut Renderer,
) -> Result<Texture> {
    let fmt = SurfaceType::R8_G8_B8_A8;
    let chan = options.channel;
    let rgba = image.rgba;
    let w = rgba.width();
    let h = rgba.height();
    if w > u32::from(u16::max_value()) || h > u32::from(u16::max_value()) {
        bail!(
            "Unsupported texture size (expected: ({}, {}), got: ({}, {})",
            u16::max_value(),
            u16::max_value(),
            w,
            h
        );
    }
    let tb = apply_options(
        TextureBuilder::new(rgba.into_raw())
            .with_format(fmt)
            .with_channel_type(chan)
            .with_size(w as u16, h as u16),
        options,
    );
    renderer
        .create_texture(tb)
        .chain_err(|| "Failed to create texture from texture data")
}

/// Aggregate texture format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TextureFormat {
    /// Jpeg
    Jpg,
    /// Png
    Png,
    /// Bmp
    Bmp,
    /// Tga
    Tga,
}

impl SimpleFormat<Texture> for TextureFormat {
    const NAME: &'static str = "TextureFormat";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        match *self {
            TextureFormat::Jpg => SimpleFormat::import(&JpgFormat, bytes, options),
            TextureFormat::Png => SimpleFormat::import(&PngFormat, bytes, options),
            TextureFormat::Bmp => SimpleFormat::import(&BmpFormat, bytes, options),
            TextureFormat::Tga => SimpleFormat::import(&TgaFormat, bytes, options),
        }
    }
}

mod serde_helper {
    use crate::tex::{FilterMethod, WrapMode};

    use super::SamplerInfo;

    fn default_filter() -> FilterMethod {
        FilterMethod::Trilinear
    }

    fn default_wrap() -> WrapMode {
        WrapMode::Clamp
    }

    pub fn default_sampler() -> SamplerInfo {
        SamplerInfo::new(default_filter(), default_wrap())
    }

    pub fn default_mip_levels() -> u8 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::TextureData;

    #[test]
    fn texture_data_from_f32_3() {
        match TextureData::from([0.25, 0.50, 0.75]) {
            TextureData::Rgba(color, _) => {
                assert_eq!(color, [0.25, 0.50, 0.75, 1.0]);
            }
            _ => panic!("Expected [f32; 3] to turn into TextureData::Rgba"),
        }
    }
}
