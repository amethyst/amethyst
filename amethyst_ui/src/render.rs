use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use amethyst_assets::{PrefabData, Format, Asset};

pub trait UiRenderer: Send + Sync + 'static + Clone {
    type Texture: Send + Sync + Asset + Clone + Debug;
    fn color_texture(color: [f32; 4]) -> <Self::Texture as Asset>::Data;
}

pub trait TexturePrefab<R, F>: Send + Sync + 'static +
    Clone + Serialize + for<'de> Deserialize<'de> + for<'a> PrefabData<'a>
where
    R: UiRenderer,
    F: Format<R::Texture, Options = ()>,
{}
