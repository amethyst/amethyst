use crate::types::Texture;
use amethyst_assets::{
    AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter, SimpleFormat,
};
use amethyst_core::ecs::{Entity, Read, ReadExpect};
use amethyst_error::Error;
use rendy::{
    hal::Backend,
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

    /// Load file with format
    File(String, F, F::Options),

    /// Clone handle only
    #[serde(skip)]
    Handle(Handle<Texture<B>>),
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
