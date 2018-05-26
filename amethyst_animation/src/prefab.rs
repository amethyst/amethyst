use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use amethyst_assets::{AssetStorage, Handle, Loader, PrefabData};
use amethyst_core::specs::error::Error;
use amethyst_core::specs::prelude::{Entity, Read, ReadExpect, Write, WriteStorage};
use serde::{Deserialize, Serialize};

use {Animation, AnimationHierarchy, AnimationSampling, AnimationSet, RestState, Sampler};

/// `PrefabData` for loading a single `Animation`
///
/// This should be used primarily from inside other `PrefabData`, because this will not place
/// anything on the `Entity`, it will only return a `Handle<Animation>` when loaded.
///
/// ### Type parameters
///
/// - `T`: The animatable `Component`
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AnimationPrefab<T>
where
    T: AnimationSampling,
    T::Channel: for<'a> Deserialize<'a> + Serialize,
    T::Primitive: for<'a> Deserialize<'a> + Serialize,
{
    /// All samplers in the `Animation`
    pub samplers: Vec<(usize, T::Channel, Sampler<T::Primitive>)>,
}

impl<'a, T> PrefabData<'a> for AnimationPrefab<T>
where
    T: AnimationSampling,
    T::Channel: for<'b> Deserialize<'b> + Serialize,
    T::Primitive: for<'b> Deserialize<'b> + Serialize,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Sampler<T::Primitive>>>,
        Read<'a, AssetStorage<Animation<T>>>,
    );
    type Result = Handle<Animation<T>>;

    fn load_prefab(
        &self,
        _: Entity,
        &mut (ref loader, ref sampler_storage, ref animation_storage): &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<Handle<Animation<T>>, Error> {
        let animation = Animation::<T> {
            nodes: self.samplers
                .iter()
                .map(|(node_index, channel, sampler)| {
                    (
                        *node_index,
                        channel.clone(),
                        loader.load_from_data(sampler.clone(), (), sampler_storage),
                    )
                })
                .collect(),
        };
        Ok(loader.load_from_data(animation, (), animation_storage))
    }
}

/// `PrefaData` for loading `Animation`s as part of an `AnimationSet`.
///
/// ### Type parameters
///
/// - `I`: Id type
/// - `T`: The animatable `Component`
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct AnimationSetPrefab<I, T>
where
    T: AnimationSampling,
    T::Channel: for<'a> Deserialize<'a> + Serialize,
    T::Primitive: Debug + for<'a> Deserialize<'a> + Serialize,
{
    /// Pairs of `Id` and `Animation`
    pub animations: Vec<(I, AnimationPrefab<T>)>,
}

impl<'a, I, T> PrefabData<'a> for AnimationSetPrefab<I, T>
where
    T: AnimationSampling,
    T::Channel: for<'b> Deserialize<'b> + Serialize,
    T::Primitive: Debug + for<'b> Deserialize<'b> + Serialize,
    I: Clone + Hash + Eq + Send + Sync + 'static,
{
    type SystemData = (
        Write<'a, AnimationSet<I, T>>,
        <AnimationPrefab<T> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        for (id, animation_prefab) in &self.animations {
            system_data.0.insert(
                id.clone(),
                animation_prefab.load_prefab(entity, &mut system_data.1, entities)?,
            );
        }
        Ok(())
    }
}

impl<'a, T> PrefabData<'a> for RestState<T>
where
    T: AnimationSampling + Clone,
{
    type SystemData = WriteStorage<'a, RestState<T>>;
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), Error> {
        storage.insert(entity, self.clone()).map(|_| ())
    }
}

/// `PrefabData` for loading `AnimationHierarchy`.
///
/// ### Type parameters
///
/// - `T`: The animatable `Component`
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct AnimationHierarchyPrefab<T> {
    pub nodes: Vec<(usize, usize)>,
    _m: PhantomData<T>,
}

impl<'a, T> PrefabData<'a> for AnimationHierarchyPrefab<T>
where
    T: AnimationSampling,
{
    type SystemData = WriteStorage<'a, AnimationHierarchy<T>>;
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        entities: &[Entity],
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
            .map(|_| ())
    }
}

/// `PrefaData` for full animation support
///
/// ### Type parameters
///
/// - `I`: Id type of `Animation`s in `AnimationSet`s
/// - `T`: The animatable `Component`
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AnimatablePrefab<I, T>
where
    T: AnimationSampling,
    T::Channel: for<'a> Deserialize<'a> + Serialize,
    T::Primitive: Debug + for<'a> Deserialize<'a> + Serialize,
{
    /// Place an `AnimationSet` on the `Entity`
    pub animation_set: Option<AnimationSetPrefab<I, T>>,
    /// Place an `AnimationHierarchy` on the `Entity`
    pub hierarchy: Option<AnimationHierarchyPrefab<T>>,
    /// Place a `RestState` on the `Entity`
    pub rest_state: Option<RestState<T>>,
}

impl<'a, I, T> PrefabData<'a> for AnimatablePrefab<I, T>
where
    T: AnimationSampling + Clone,
    T::Channel: for<'b> Deserialize<'b> + Serialize,
    T::Primitive: Debug + for<'b> Deserialize<'b> + Serialize,
    I: Clone + Hash + Eq + Send + Sync + 'static,
{
    type SystemData = (
        <AnimationSetPrefab<I, T> as PrefabData<'a>>::SystemData,
        <AnimationHierarchyPrefab<T> as PrefabData<'a>>::SystemData,
        <RestState<T> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut <Self as PrefabData>::SystemData,
        entities: &[Entity],
    ) -> Result<<Self as PrefabData>::Result, Error> {
        if let Some(ref prefab) = self.animation_set {
            prefab.load_prefab(entity, &mut system_data.0, entities)?;
        }
        if let Some(ref prefab) = self.hierarchy {
            prefab.load_prefab(entity, &mut system_data.1, entities)?;
        }
        if let Some(ref prefab) = self.rest_state {
            prefab.load_prefab(entity, &mut system_data.2, entities)?;
        }
        Ok(())
    }
}
