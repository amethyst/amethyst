use crate::types::Texture;
use amethyst_assets::{
    AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter, SimpleFormat,
};
use amethyst_core::ecs::{Entity, Read, ReadExpect};
use amethyst_error::Error;
use rendy::{
    hal::{self, Backend, image::{Filter, Kind, ViewKind, Size}},
    texture::{
        image::{load_from_image, ImageTextureConfig},
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
pub enum TexturePrefab<B: Backend, F>
where
    F: Format<Texture<B>>,
{
    /// Texture data
    Data(TextureBuilder<'static>),

    // Generate texture
    Generate(TextureGenerator),

    /// Load file with format
    File(String, F, F::Options),

    /// Clone handle only
    #[serde(skip)]
    Handle(Handle<Texture<B>>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TextureGenerator {
    Rgba(f32, f32, f32, f32),
    RgbaCorners([f32; 16], Filter),
}

impl TextureGenerator {
    fn generate(&self) -> (Vec<u8>, Size, Filter) {
        fn float_to_byte(float: &f32) -> u8 {
            (float * 255.0).max(0.0).min(255.0) as u8
        }
        match self {
            TextureGenerator::Rgba(red, green, blue, alpha) =>
            (
                vec![
                    float_to_byte(red),
                    float_to_byte(green),
                    float_to_byte(blue),
                    float_to_byte(alpha),
                ],
                1,
                Filter::Nearest,
            ),
            TextureGenerator::RgbaCorners(corners, filter) => (
                corners.iter().map(float_to_byte).collect(),
                2,
                *filter
            ),
        }
    }
    fn data(&self) -> TextureBuilder<'static> {
        let (data, size, filter) = self.generate();
        TextureBuilder::new()
            .with_kind(Kind::D2(size,size,1,1))
            .with_view_kind(ViewKind::D2)
            .with_data_width(size)
            .with_data_height(size)
            .with_filter(filter)
            .with_raw_data(data, hal::format::Format::Rgba8Srgb)
    }
}

impl<'a, B: Backend, F> PrefabData<'a> for TexturePrefab<B, F>
where
    F: Format<Texture<B>> + Clone + Sync,
    F::Options: Clone,
{
    type SystemData = (ReadExpect<'a, Loader>, Read<'a, AssetStorage<Texture<B>>>);

    type Result = Handle<Texture<B>>;

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<Handle<Texture<B>>, Error> {
        let handle = match *self {
            TexturePrefab::Data(ref data) => {
                system_data
                    .0
                    .load_from_data(data.clone(), (), &system_data.1)
            }

            TexturePrefab::Generate(ref generator) => {
                let data = generator.data();
                system_data
                    .0
                    .load_from_data(data, (), &system_data.1)
            },

            TexturePrefab::File(..) => unreachable!(),

            TexturePrefab::Handle(ref handle) => handle.clone(),
        };
        Ok(handle)
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let handle = match *self {
            TexturePrefab::Data(ref data) => Some(system_data.0.load_from_data(
                data.clone(),
                progress,
                &system_data.1,
            )),

            TexturePrefab::Generate(ref generator) => {
                let data = generator.data();
                Some(system_data.0.load_from_data(
                    data,
                    progress,
                    &system_data.1,
                ))
            },

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
