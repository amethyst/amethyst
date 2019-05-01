use std::any::Any;
use serde::de::DeserializeOwned;
use amethyst_assets::{PrefabData, Format};

pub trait UiRenderer: Any + Send + Sync {
    type Texture: Any + Send + Sync;
}

pub trait TexturePrefab<R, F>: Any + Send + Sync + 'static +
    Clone + DeserializeOwned + PrefabData<'_>
where
    R: UiRenderer,
    F: Format<R::Texture, Options = ()>,
{}
