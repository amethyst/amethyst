use std::{any::Any, fmt::Debug};
use serde::Deserialize;
use amethyst_assets::{PrefabData, Format, Handle};
use amethyst_core::ecs::prelude::{Component, DenseVecStorage};

pub trait UiRenderer: Any + Send + Sync {
    type Texture: Any + Send + Sync;
}

pub trait TextureFormat<R: UiRenderer>: Any + Send + Sync +
    Format<R::Texture> {}

pub trait TextureMetadata<R: UiRenderer>: Any + Send + Sync +
    Debug + Clone {}

pub trait TexturePrefab<'a, R, F>: Any + Send + Sync +
    Clone + Deserialize<'a> + PrefabData<'a>
where
    R: UiRenderer,
    F: TextureFormat<R>,
{}
