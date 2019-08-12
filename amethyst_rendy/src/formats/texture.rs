//! Texture formats implementation.
use crate::types::{Texture, TextureData};
use amethyst_assets::{
    AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter, SerializableFormat,
};
use amethyst_core::ecs::{Entity, Read, ReadExpect};
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

/// Image format description newtype wrapper for `ImageTextureConfig` from rendy.
///
/// # Example Usage
/// ```ignore
///
///    let loader = res.fetch_mut::<Loader>();
///    let texture_storage = res.fetch_mut::<AssetStorage<Texture>>();
///
///    let texture_builder = TextureBuilder::new()
///        .with_data_width(handle.width)
///        .with_data_height(handle.height)
///        .with_kind(image::Kind::D2(handle.width, handle.height, 1, 1))
///        .with_view_kind(image::ViewKind::D2)
///        .with_sampler_info(SamplerInfo {
///        min_filter: Filter::Linear,
///        mag_filter: Filter::Linear,
///        mip_filter: Filter::Linear,
///        wrap_mode: (WrapMode::Clamp, WrapMode::Clamp, WrapMode::Clamp),
///        lod_bias: 0.0.into(),
///        lod_range: std::ops::Range {
///            start: 0.0.into(),
///            end: 1000.0.into(),
///        },
///        comparison: None,
///        border: PackedColor(0),
///        anisotropic: Anisotropic::Off,
///        })
///        .with_raw_data(handle.pixels, Format::Rgba8Unorm);
///
///    let tex: Handle<Texture> = loader.load_from_data(TextureData(texture_builder), (), &texture_storage);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImageFormat(pub ImageTextureConfig);

impl Default for ImageFormat {
    fn default() -> Self {
        use rendy::{
            hal::image::{Anisotropic, PackedColor, SamplerInfo, WrapMode},
            texture::image::{Repr, TextureKind},
        };

        ImageFormat(ImageTextureConfig {
            format: None,
            repr: Repr::Srgb,
            kind: TextureKind::D2,
            sampler_info: SamplerInfo {
                min_filter: Filter::Nearest,
                mag_filter: Filter::Nearest,
                mip_filter: Filter::Nearest,
                wrap_mode: (WrapMode::Tile, WrapMode::Tile, WrapMode::Tile),
                lod_bias: 0.0.into(),
                lod_range: std::ops::Range {
                    start: 0.0.into(),
                    end: 1000.0.into(),
                },
                comparison: None,
                border: PackedColor(0),
                anisotropic: Anisotropic::Off,
                normalized: true,
            },
            generate_mips: false,
            premultiply_alpha: true,
        })
    }
}

amethyst_assets::register_format_type!(TextureData);

amethyst_assets::register_format!("IMAGE", ImageFormat as TextureData);
impl Format<TextureData> for ImageFormat {
    fn name(&self) -> &'static str {
        "IMAGE"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<TextureData, Error> {
        load_from_image(std::io::Cursor::new(&bytes), self.0.clone())
            .map(|builder| builder.into())
            .map_err(|e| e.compat().into())
    }
}

/// `PrefabData` for loading `Texture`s.
///
/// Will not add any `Component`s to the `Entity`, will only return a `Handle`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound = "")]
pub enum TexturePrefab {
    /// Texture data
    Data(TextureData),

    /// Generate texture
    Generate(TextureGenerator),
    /// Load file with format
    File(String, Box<dyn SerializableFormat<TextureData>>),

    /// Clone handle only
    #[serde(skip)]
    Handle(Handle<Texture>),
    /// Placeholder during loading
    #[serde(skip)]
    Placeholder,
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
        .with_sampler_info(hal::image::SamplerInfo::new(
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
            TextureGenerator::SrgbaCorners(corners, filter) => simple_builder::<Rgba8Srgb>(
                corners
                    .iter()
                    .map(|(red, green, blue, alpha)| {
                        palette::Srgba::new(*red, *green, *blue, *alpha).into()
                    })
                    .collect(),
                2,
                filter,
            ),
        }
    }
}

impl<'a> PrefabData<'a> for TexturePrefab {
    type SystemData = (ReadExpect<'a, Loader>, Read<'a, AssetStorage<Texture>>);

    type Result = Handle<Texture>;

    fn add_to_entity(
        &self,
        _: Entity,
        _: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<Handle<Texture>, Error> {
        let handle = match *self {
            TexturePrefab::Handle(ref handle) => handle.clone(),
            _ => unreachable!(),
        };
        Ok(handle)
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        (loader, storage): &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (ret, next) = match std::mem::replace(self, TexturePrefab::Placeholder) {
            TexturePrefab::Data(data) => {
                let handle = loader.load_from_data(data, progress, storage);
                (true, TexturePrefab::Handle(handle))
            }
            TexturePrefab::Generate(generator) => {
                let handle = loader.load_from_data(generator.data(), progress, storage);
                (true, TexturePrefab::Handle(handle))
            }
            TexturePrefab::File(name, format) => {
                let handle = loader.load(name, format, progress, storage);
                (true, TexturePrefab::Handle(handle))
            }
            slot => (false, slot),
        };
        *self = next;
        Ok(ret)
    }
}
