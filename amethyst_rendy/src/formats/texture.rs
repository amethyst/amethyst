//! Texture formats implementation.
use amethyst_assets::Format;
use amethyst_error::Error;
use rendy::{
    hal::{
        self,
        image::{Filter, Kind, Size, ViewKind},
    },
    texture::{
        image::{load_from_image, ImageTextureConfig},
        pixel::{AsPixel, Rgba8Srgb},
        TextureBuilder,
    },
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::types::TextureData;

/// Image format description newtype wrapper for `ImageTextureConfig` from rendy.
///
/// # Example Usage
/// ```
/// use amethyst::{
///     assets::{AssetStorage, DefaultLoader, Handle, Loader, ProcessingQueue},
///     ecs::Resources,
///     error::Error,
///     renderer::{
///         rendy::{
///             hal::{
///                 format::Format,
///                 image::{
///                     Filter, Kind, Lod, PackedColor, SamplerDesc, Size, ViewKind, WrapMode,
///                 },
///             },
///             texture::{
///                 image::{load_from_image, ImageTextureConfig, Repr, TextureKind},
///                 pixel::{AsPixel, Rgba8Srgb},
///                 TextureBuilder,
///             },
///         },
///         types::TextureData,
///         Texture,
///     },
/// };
/// # let mut resources = Resources::default();
/// # let mut loader = DefaultLoader::default();
/// let texture_storage = resources.get_or_default::<ProcessingQueue<TextureData>>();
/// struct Image {
///     pixels: Vec<u8>,
///     width: u32,
///     height: u32,
/// }
///
/// let handle = Image {
///     pixels: vec![],
///     width: 1,
///     height: 1,
/// };
///
/// let texture_builder = TextureBuilder::new()
///     .with_data_width(handle.width)
///     .with_data_height(handle.height)
///     .with_kind(Kind::D2(handle.width, handle.height, 1, 1))
///     .with_view_kind(ViewKind::D2)
///     .with_sampler_info(SamplerDesc {
///         min_filter: Filter::Linear,
///         mag_filter: Filter::Linear,
///         mip_filter: Filter::Linear,
///         wrap_mode: (WrapMode::Clamp, WrapMode::Clamp, WrapMode::Clamp),
///         lod_bias: Lod(0.0),
///         lod_range: std::ops::Range {
///             start: Lod(0.0),
///             end: Lod(1000.0),
///         },
///         comparison: None,
///         border: PackedColor(0),
///         normalized: true,
///         anisotropy_clamp: None,
///     })
///     .with_raw_data(handle.pixels, Format::Rgba8Unorm);
///
/// let tex: Handle<Texture> =
///     loader.load_from_data(TextureData(texture_builder), (), &texture_storage);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "79f58dea-e7c7-4305-a116-cd8313c04784"]
pub struct ImageFormat(pub ImageTextureConfig);

impl Default for ImageFormat {
    fn default() -> Self {
        use rendy::{
            hal::image::{Lod, PackedColor, SamplerDesc, WrapMode},
            texture::image::{Repr, TextureKind},
        };

        ImageFormat(ImageTextureConfig {
            format: None,
            repr: Repr::Srgb,
            kind: TextureKind::D2,
            sampler_info: SamplerDesc {
                min_filter: Filter::Nearest,
                mag_filter: Filter::Nearest,
                mip_filter: Filter::Nearest,
                wrap_mode: (WrapMode::Tile, WrapMode::Tile, WrapMode::Tile),
                lod_bias: Lod(0.0),
                lod_range: std::ops::Range {
                    start: Lod(0.0),
                    end: Lod(1000.0),
                },
                comparison: None,
                border: PackedColor(0),
                normalized: true,
                anisotropy_clamp: None,
            },
            generate_mips: false,
            premultiply_alpha: true,
        })
    }
}

amethyst_assets::register_importer!(".jpg", ImageFormat);
amethyst_assets::register_importer!(".png", ImageFormat);
amethyst_assets::register_importer!(".tga", ImageFormat);
amethyst_assets::register_importer!(".bmp", ImageFormat);
impl Format<TextureData> for ImageFormat {
    fn name(&self) -> &'static str {
        "IMAGE"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<TextureData, Error> {
        load_from_image(std::io::Cursor::new(&bytes), self.0.clone())
            .map(|builder| builder.into())
            .map_err(|e| e.into())
    }
}

/// Provides enum variant typecasting of texture data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TextureGenerator {
    /// Srgba value (`f32` * 4)
    Srgba(f32, f32, f32, f32),
    /// LinearRgba value (`f32` * 4)
    LinearRgba(f32, f32, f32, f32),
    /// SrgbaCorners value (`f32` * 4) + [Filter]
    SrgbaCorners([(f32, f32, f32, f32); 4], Filter),
}

fn simple_builder<A: AsPixel>(data: Vec<A>, size: Size, filter: Filter) -> TextureData {
    TextureBuilder::new()
        .with_kind(Kind::D2(size, size, 1, 1))
        .with_view_kind(ViewKind::D2)
        .with_data_width(size)
        .with_data_height(size)
        .with_sampler_info(hal::image::SamplerDesc::new(
            filter,
            hal::image::WrapMode::Clamp,
        ))
        .with_data(data)
        .into()
}

impl TextureGenerator {
    /// Converts the provided texture enum variant values in a generic TextureData format.
    pub fn data(&self) -> TextureData {
        use palette::{LinSrgba, Srgba};
        use rendy::texture::palette::{load_from_linear_rgba, load_from_srgba};
        match *self {
            TextureGenerator::Srgba(red, green, blue, alpha) => {
                load_from_srgba(Srgba::new(red, green, blue, alpha)).into()
            }
            TextureGenerator::LinearRgba(red, green, blue, alpha) => {
                load_from_linear_rgba(LinSrgba::new(red, green, blue, alpha)).into()
            }
            //TextureGenerator::LinearRgbaFloat(red, green, blue, alpha) => load_from_linear_rgba_f32(
            //    LinSrgba::new(red, green, blue, alpha)
            //),
            TextureGenerator::SrgbaCorners(corners, filter) => {
                simple_builder::<Rgba8Srgb>(
                    corners
                        .iter()
                        .map(|(red, green, blue, alpha)| {
                            palette::Srgba::new(*red, *green, *blue, *alpha).into()
                        })
                        .collect(),
                    2,
                    filter,
                )
            }
        }
    }
}
