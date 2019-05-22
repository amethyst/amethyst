use std::{
    error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::Hash,
    marker::PhantomData,
};

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use amethyst_assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter};
use amethyst_core::ecs::prelude::{Entity, Read, ReadExpect, WriteStorage};
use amethyst_derive::PrefabData;
use amethyst_error::Error;

use crate::{Animation, AnimationHierarchy, AnimationSampling, AnimationSet, RestState, Sampler};

/// `PrefabData` for loading a single `Animation`
///
/// This should be used primarily from inside other `PrefabData`, because this will not place
/// anything on the `Entity`, it will only return a `Handle<Animation>` when loaded.
///
/// ### Type parameters
///
/// - `T`: The animatable `Component`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(bound(
    serialize = "T::Channel: Serialize, T::Primitive: Serialize",
    deserialize = "T::Channel: Deserialize<'de>, T::Primitive: Deserialize<'de>",
))]
pub struct AnimationPrefab<T>
where
    T: AnimationSampling,
{
    /// All samplers in the `Animation`
    pub samplers: Vec<(usize, T::Channel, Sampler<T::Primitive>)>,
    #[serde(skip, default = "default_handle")]
    handle: Option<Handle<Animation<T>>>,
}

fn default_handle<T>() -> Option<Handle<Animation<T>>>
where
    T: AnimationSampling,
{
    None
}

impl<T> Default for AnimationPrefab<T>
where
    T: AnimationSampling,
{
    fn default() -> Self {
        AnimationPrefab {
            samplers: Vec::default(),
            handle: None,
        }
    }
}

#[derive(Debug)]
pub struct MissingAssetHandle;

impl Display for MissingAssetHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "AnimationPrefab was not populated with an asset handle prior to calling load_prefab."
        )
    }
}

impl error::Error for MissingAssetHandle {}

impl<'a, T> PrefabData<'a> for AnimationPrefab<T>
where
    T: AnimationSampling,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Sampler<T::Primitive>>>,
        Read<'a, AssetStorage<Animation<T>>>,
    );
    type Result = Handle<Animation<T>>;

    fn add_to_entity(
        &self,
        _: Entity,
        _: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<Handle<Animation<T>>, Error> {
        Ok(self
            .handle
            .as_ref()
            .cloned()
            .ok_or_else(|| MissingAssetHandle)?)
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        &mut (ref loader, ref sampler_storage, ref animation_storage): &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let animation = Animation::<T> {
            nodes: self
                .samplers
                .iter()
                .map(|(node_index, channel, sampler)| {
                    (
                        *node_index,
                        channel.clone(),
                        loader.load_from_data(sampler.clone(), &mut *progress, sampler_storage),
                    )
                })
                .collect(),
        };
        self.handle = Some(loader.load_from_data(animation, progress, animation_storage));
        Ok(true)
    }
}

/// `PrefabData` for loading `Animation`s as part of an `AnimationSet`.
///
/// ### Type parameters
///
/// - `I`: Id type
/// - `T`: The animatable `Component`
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(bound(
    serialize = "I: Serialize, AnimationPrefab<T>: Serialize",
    deserialize = "I: Deserialize<'de>, AnimationPrefab<T>: Deserialize<'de>",
))]
pub struct AnimationSetPrefab<I, T>
where
    T: AnimationSampling,
    T::Primitive: Debug,
{
    /// Pairs of `Id` and `Animation`
    pub animations: Vec<(I, AnimationPrefab<T>)>,
}

impl<'a, I, T> PrefabData<'a> for AnimationSetPrefab<I, T>
where
    T: AnimationSampling,
    T::Primitive: Debug,
    I: Clone + Hash + Eq + Send + Sync + 'static,
{
    type SystemData = (
        WriteStorage<'a, AnimationSet<I, T>>,
        <AnimationPrefab<T> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        let set = system_data
            .0
            .entry(entity)?
            .or_insert_with(AnimationSet::default);
        for (id, animation_prefab) in &self.animations {
            set.insert(
                id.clone(),
                animation_prefab.add_to_entity(entity, &mut system_data.1, entities, &[])?,
            );
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let mut ret = false;
        for (_, animation_prefab) in &mut self.animations {
            if animation_prefab.load_sub_assets(progress, &mut system_data.1)? {
                ret = true;
            }
        }
        Ok(ret)
    }
}

/// `PrefabData` for loading `AnimationHierarchy`.
///
/// ### Type parameters
///
/// - `T`: The animatable `Component`
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(bound = "")]
pub struct AnimationHierarchyPrefab<T> {
    /// A vec of the node index and the entity index.
    pub nodes: Vec<(usize, usize)>,
    _m: PhantomData<T>,
}

impl<'a, T> PrefabData<'a> for AnimationHierarchyPrefab<T>
where
    T: AnimationSampling,
{
    type SystemData = WriteStorage<'a, AnimationHierarchy<T>>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage
            .insert(
                entity,
                AnimationHierarchy::new_many(
                    self.nodes
                        .iter()
                        .map(|(node_index, entity_index)| (*node_index, entities[*entity_index]))
                        .collect(),
                ),
            )
            .map(|_| ())?;

        Ok(())
    }
}

/// `PrefabData` for full animation support
///
/// ### Type parameters
///
/// - `I`: Id type of `Animation`s in `AnimationSet`s
/// - `T`: The animatable `Component`
#[derive(Derivative, Clone, Debug, Deserialize, Serialize, PrefabData)]
#[serde(
    default,
    bound(
        serialize = "T: Serialize, AnimationSetPrefab<I, T>: Serialize",
        deserialize = "T: Deserialize<'de>, AnimationSetPrefab<I, T>: Deserialize<'de>",
    )
)]
#[derivative(Default(bound = ""))]
pub struct AnimatablePrefab<I, T>
where
    T: AnimationSampling + Clone,
    T::Primitive: Debug,
    I: Clone + Hash + Eq + Send + Sync + 'static,
{
    /// Place an `AnimationSet` on the `Entity`
    pub animation_set: Option<AnimationSetPrefab<I, T>>,
    /// Place an `AnimationHierarchy` on the `Entity`
    pub hierarchy: Option<AnimationHierarchyPrefab<T>>,
    /// Place a `RestState` on the `Entity`
    pub rest_state: Option<RestState<T>>,
}
