use crate::types::Texture;
use amethyst_assets::{
    AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter, SimpleFormat,
};
use amethyst_core::ecs::{Entity, Read, ReadExpect};
use amethyst_error::Error;
use rendy::{
    hal::{
        self,
        image::{Filter, Kind, Size, ViewKind},
        Backend,
    },
    texture::{
        image::{load_from_image, ImageTextureConfig},
        pixel::{AsPixel, Rgba8Srgb},
        TextureBuilder,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ImageFormat;

impl<B: Backend> SimpleFormat<Texture<B>> for ImageFormat {
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

/// `PrefabData` for loading `Texture`s.
///
/// Will not add any `Component`s to the `Entity`, will only return a `Handle`
///
/// ### Type parameters:
///
/// - `B`: `Backend` parameter to use for `Texture<B>` type
/// - `F`: `Format` to use for loading the `Texture`s from file
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound = "")]
pub enum TexturePrefab<B: Backend, F = ImageFormat>
where
    F: Format<Texture<B>>,
{
    /// Texture data
    Data(TextureBuilder<'static>),

    // Generate texture
    Generate(TextureGenerator),
    #[serde(bound(
        serialize = "F: Serialize, F::Options: Serialize",
        deserialize = "F: Deserialize<'de>, F::Options: Deserialize<'de>",
    ))]
    /// Load file with format
    File(String, F, F::Options),

    /// Clone handle only
    #[serde(skip)]
    Handle(Handle<Texture<B>>),
    /// Placeholder during loading
    #[serde(skip)]
    Placeholder,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TextureGenerator {
    Srgba(f32, f32, f32, f32),
    LinearRgba(f32, f32, f32, f32),
    //LinearRgbaFloat(f32, f32, f32, f32),
    SrgbaCorners([(f32, f32, f32, f32); 4], Filter),
}

fn simple_builder<A: AsPixel>(data: Vec<A>, size: Size, filter: Filter) -> TextureBuilder<'static> {
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
}

impl TextureGenerator {
    fn data(&self) -> TextureBuilder<'static> {
        use palette::{LinSrgba, Srgba};
        use rendy::texture::palette::{load_from_linear_rgba, load_from_srgba};
        match *self {
            TextureGenerator::Srgba(red, green, blue, alpha) => {
                load_from_srgba(Srgba::new(red, green, blue, alpha))
            }
            TextureGenerator::LinearRgba(red, green, blue, alpha) => {
                load_from_linear_rgba(LinSrgba::new(red, green, blue, alpha))
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

impl<'a, B: Backend, F> PrefabData<'a> for TexturePrefab<B, F>
where
    F: Format<Texture<B>> + Sync,
{
    type SystemData = (ReadExpect<'a, Loader>, Read<'a, AssetStorage<Texture<B>>>);

    type Result = Handle<Texture<B>>;

    fn add_to_entity(
        &self,
        _: Entity,
        _: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<Handle<Texture<B>>, Error> {
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
            TexturePrefab::File(name, format, options) => {
                let handle = loader.load(name.as_ref(), format, options, progress, storage);
                (true, TexturePrefab::Handle(handle))
            }
            slot => (false, slot),
        };
        *self = next;
        Ok(ret)
    }
}
