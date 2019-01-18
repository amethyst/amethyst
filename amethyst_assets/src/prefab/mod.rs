use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use shred_derive::SystemData;

use amethyst_core::specs::prelude::{
    Component, DenseVecStorage, Entity, FlaggedStorage, Read, ReadExpect, SystemData, WriteStorage,
};

use crate::{Asset, AssetStorage, Format, Handle, Loader, Progress, ProgressCounter};

pub use self::system::PrefabLoaderSystem;
pub use amethyst_core::specs::error::Error as PrefabError;

mod impls;
mod system;

/// Trait for loading a prefabs data for a single entity
pub trait PrefabData<'a> {
    /// `SystemData` needed to perform the load
    type SystemData: SystemData<'a>;

    /// The result type returned by the load operation
    type Result;

    /// Add the data for this prefab onto the given `Entity`
    ///
    /// This can also be used to load resources, the recommended way of doing so is to put the
    /// resources on the main `Entity` of the `Prefab`
    ///
    /// ### Parameters:
    ///
    /// - `entity`: `Entity` to load components on, or the root `Entity` for the resource scenario
    /// - `system_data`: `SystemData` needed to do the loading
    /// - `entities`: Some components need access to the entities that was created as part of the
    ///               full prefab, for linking purposes, so this contains all those `Entity`s.
    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<Self::Result, PrefabError>;

    /// Trigger asset loading for any sub assets.
    ///
    /// ### Parameters:
    ///
    /// - `progress`: Progress structure that needs to be used for tracking progress of sub loads
    /// - `system_data`: `SystemData` for the prefab
    ///
    /// ### Returns
    ///
    /// - `Err(error)` - if an `Error` occurs
    /// - `Ok(false)` - if no sub assets need loading
    /// - `Ok(true)` - if sub assets need loading, in this case the sub asset load must be added to
    ///                the given progress tracker
    ///
    /// ### Type parameters:
    ///
    /// - `P`: Progress tracker
    fn load_sub_assets(
        &mut self,
        _progress: &mut ProgressCounter,
        _system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        Ok(false)
    }
}

/// Main `Prefab` structure, containing all data loaded in a single prefab.
///
/// Contains a list prefab data for the entities affected by the prefab. The first entry in the
/// `entities` list will be applied to the main `Entity` the `Handle` is placed on, and all other
/// entries will trigger creation of a new entity. Note that the parent index is ignored for the
/// first entry in the list.
///
/// The recommended way of loading resources is to place them on the main `Entity`.
///
/// ### Example:
///
/// If the prefab contains 3 new entities `A`, `B` and `C`, and the main `Entity` that the `Handle`
/// is placed on is `E`, and we want the graph to be `A -> E`, `B -> E`, `C -> B` (parent links),
/// the data will be as follows:
///
/// ```rust,ignore
/// Prefab {
///     entities: vec![
///         PrefabEntity { parent: None /* not used */, .. },
///         PrefabEntity { parent: Some(0), .. },
///         PrefabEntity { parent: Some(0), .. },
///         PrefabEntity { parent: Some(2), .. },
///     ],
/// }
/// ```
///
/// ### Type parameters:
///
/// - `T`: `PrefabData`
#[derive(Default, Deserialize, Serialize)]
pub struct Prefab<T> {
    #[serde(skip)]
    tag: Option<u64>,
    entities: Vec<PrefabEntity<T>>,
    #[serde(skip)]
    counter: Option<ProgressCounter>,
}

/// Prefab data container for a single entity
///
/// ### Type parameters:
///
/// - `T`: `PrefabData`
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PrefabEntity<T> {
    parent: Option<usize>,
    data: Option<T>,
}

impl<T> Default for PrefabEntity<T> {
    fn default() -> Self {
        PrefabEntity::new(None, None)
    }
}

impl<T> PrefabEntity<T> {
    /// New prefab entity
    pub fn new(parent: Option<usize>, data: Option<T>) -> Self {
        PrefabEntity { parent, data }
    }

    /// Set parent index
    pub fn set_parent(&mut self, parent: usize) {
        self.parent = Some(parent);
    }

    /// Set data
    pub fn set_data(&mut self, data: T) {
        self.data = Some(data);
    }

    /// Get immutable access to the data
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    /// Get mutable access to the data
    pub fn data_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut()
    }

    /// Get mutable access to the data
    ///
    /// If Option is `None`, insert a default entry
    pub fn data_or_default(&mut self) -> &mut T
    where
        T: Default,
    {
        self.data.get_or_insert_with(T::default)
    }

    /// Trigger sub asset loading for the prefab entity
    pub fn load_sub_assets<'a>(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut <T as PrefabData<'a>>::SystemData,
    ) -> Result<bool, PrefabError>
    where
        T: PrefabData<'a>,
    {
        if let Some(ref mut data) = self.data {
            data.load_sub_assets(progress, system_data)
        } else {
            Ok(false)
        }
    }
}

impl<T> Prefab<T> {
    /// Create new empty prefab
    pub fn new() -> Self {
        Prefab {
            tag: None,
            entities: vec![PrefabEntity::default()],
            counter: None,
        }
    }

    /// Create a prefab with data for only the main `Entity`
    pub fn new_main(data: T) -> Self {
        Prefab {
            tag: None,
            entities: vec![PrefabEntity::new(None, Some(data))],
            counter: None,
        }
    }

    /// Set main `Entity` data
    pub fn main(&mut self, data: Option<T>) {
        self.entities[0].data = data;
    }

    /// Add a new entity to the prefab, with optional data and parent.
    pub fn add(&mut self, parent: Option<usize>, data: Option<T>) -> usize {
        let index = self.entities.len();
        self.entities.push(PrefabEntity::new(parent, data));
        index
    }

    /// Number of entities in the prefab, including the main entity
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Create a new entity in the prefab, with no data and no parent
    pub fn new_entity(&mut self) -> usize {
        self.add(None, None)
    }

    /// Get mutable access to the `PrefabEntity` with the given index
    pub fn entity(&mut self, index: usize) -> Option<&mut PrefabEntity<T>> {
        self.entities.get_mut(index)
    }

    /// Get immutable access to all entities in the prefab
    pub fn entities(&self) -> impl Iterator<Item = &PrefabEntity<T>> {
        self.entities.iter()
    }

    /// Get mutable access to the data in the `PrefabEntity` with the given index
    ///
    /// If data is None, this will insert a default value for `T`
    ///
    /// ### Panics
    ///
    /// If the given index do not have a `PrefabEntity`
    pub fn data_or_default(&mut self, index: usize) -> &mut T
    where
        T: Default,
    {
        self.entities[index].data_or_default()
    }

    /// Check if sub asset loading have been triggered
    pub fn loading(&self) -> bool {
        self.counter.is_some()
    }

    /// Get the `ProgressCounter` for the sub asset loading.
    ///
    /// ### Panics
    ///
    /// If sub asset loading has not been triggered.
    pub fn progress(&self) -> &ProgressCounter {
        self.counter
            .as_ref()
            .expect("Sub asset loading has not been triggered")
    }

    /// Trigger sub asset loading for the asset
    pub fn load_sub_assets<'a>(
        &mut self,
        system_data: &mut <T as PrefabData<'a>>::SystemData,
    ) -> Result<bool, PrefabError>
    where
        T: PrefabData<'a>,
    {
        let mut ret = false;
        let mut progress = ProgressCounter::default();
        for entity in &mut self.entities {
            if entity.load_sub_assets(&mut progress, system_data)? {
                ret = true;
            }
        }
        self.counter = Some(progress);
        Ok(ret)
    }
}

/// Tag placed on entities created by the prefab system.
///
/// The tag value match the tag value of the `Prefab` the `Entity` was created from.
pub struct PrefabTag<T> {
    tag: u64,
    _m: PhantomData<T>,
}

impl<T> PrefabTag<T> {
    /// Create a new tag
    pub fn new(tag: u64) -> Self {
        PrefabTag {
            tag,
            _m: PhantomData,
        }
    }

    /// Get the tag
    pub fn tag(&self) -> u64 {
        self.tag
    }
}

impl<T> Component for PrefabTag<T>
where
    T: Send + Sync + 'static,
{
    type Storage = DenseVecStorage<Self>;
}

impl<T> Asset for Prefab<T>
where
    T: Send + Sync + 'static,
{
    const NAME: &'static str = "PREFAB";
    type Data = Self;
    type HandleStorage = FlaggedStorage<Handle<Self>, DenseVecStorage<Handle<Self>>>;
}

/// Convenience `PrefabData` for loading assets of type `A` using `Format` `F`.
///
/// Will add a `Handle<A>` to the `Entity`
///
/// ### Type parameters:
///
/// - `A`: `Asset`,
/// - `F`: `Format` for loading `A`
#[derive(Clone, Deserialize, Serialize)]
pub enum AssetPrefab<A, F>
where
    A: Asset,
    F: Format<A>,
{
    /// From existing handle
    #[serde(skip)]
    Handle(Handle<A>),

    /// From file, (name, format, format options)
    File(String, F, F::Options),
}

impl<'a, A, F> PrefabData<'a> for AssetPrefab<A, F>
where
    A: Asset,
    F: Format<A> + Clone,
    F::Options: Clone,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        WriteStorage<'a, Handle<A>>,
        Read<'a, AssetStorage<A>>,
    );

    type Result = Handle<A>;

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<Handle<A>, PrefabError> {
        let handle = match *self {
            AssetPrefab::Handle(ref handle) => handle.clone(),
            AssetPrefab::File(..) => unreachable!(),
        };
        system_data.1.insert(entity, handle.clone()).map(|_| handle)
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let handle = if let AssetPrefab::File(ref name, ref format, ref options) = *self {
            Some(system_data.0.load(
                name.as_ref(),
                format.clone(),
                options.clone(),
                progress,
                &system_data.2,
            ))
        } else {
            None
        };
        if let Some(handle) = handle {
            *self = AssetPrefab::Handle(handle);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Helper structure for loading prefabs.
///
/// The recommended way of using this from `State`s is to use `world.exec`.
///
/// ### Example
///
/// ```rust,ignore
/// let prefab_handle = world.exec(|loader: PrefabLoader<SomePrefab>| {
///     loader.load("prefab.ron", RonFormat, (), ()
/// });
/// ```
#[derive(SystemData)]
pub struct PrefabLoader<'a, T>
where
    T: Send + Sync + 'static,
{
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<Prefab<T>>>,
}

impl<'a, T> PrefabLoader<'a, T>
where
    T: Send + Sync + 'static,
{
    /// Load prefab from source
    pub fn load<F, N, P>(
        &self,
        name: N,
        format: F,
        options: F::Options,
        progress: P,
    ) -> Handle<Prefab<T>>
    where
        F: Format<Prefab<T>>,
        N: Into<String>,
        P: Progress,
    {
        self.loader
            .load(name, format, options, progress, &self.storage)
    }

    /// Load prefab from explicit data
    pub fn load_from_data<P>(&self, data: Prefab<T>, progress: P) -> Handle<Prefab<T>>
    where
        P: Progress,
    {
        self.loader.load_from_data(data, progress, &self.storage)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rayon::ThreadPoolBuilder;

    use amethyst_core::{
        specs::{Builder, RunNow, World},
        GlobalTransform, Time, Transform,
    };

    use crate::Loader;

    use super::*;

    type MyPrefab = Transform;

    #[test]
    fn test_prefab_load() {
        let mut world = World::new();
        let pool = Arc::new(ThreadPoolBuilder::default().build().unwrap());
        world.add_resource(pool.clone());
        world.add_resource(Loader::new(".", pool));
        world.add_resource(Time::default());
        let mut system = PrefabLoaderSystem::<MyPrefab>::default();
        RunNow::setup(&mut system, &mut world.res);

        let prefab = Prefab::new_main(Transform::default());

        let handle = world.read_resource::<Loader>().load_from_data(
            prefab,
            (),
            &world.read_resource::<AssetStorage<Prefab<MyPrefab>>>(),
        );
        let root_entity = world.create_entity().with(handle).build();
        system.run_now(&world.res);
        assert_eq!(
            Some(&Transform::default()),
            world.read_storage().get(root_entity)
        );
        assert!(world
            .read_storage::<GlobalTransform>()
            .get(root_entity)
            .is_some());
    }
}
