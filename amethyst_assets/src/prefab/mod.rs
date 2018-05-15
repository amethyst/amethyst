pub use self::system::PrefabLoaderSystem;

use amethyst_core::specs::error::Error as SpecsError;
use amethyst_core::specs::prelude::{DenseVecStorage, Entity, Read, ReadExpect, SystemData,
                                    WriteStorage};

use error::Error;
use {Asset, AssetStorage, Format, Handle, Loader, Progress};

mod impls;
mod system;

/// Trait for loading a prefabs data for a single entity
pub trait PrefabData<'a> {
    /// `SystemData` needed to perform the load
    type SystemData: SystemData<'a>;

    /// Load the data for this prefab onto the given `Entity`
    ///
    /// ### Parameters:
    ///
    /// - `entity`: `Entity` to load components on
    /// - `system_data`: `SystemData` needed to do the loading
    /// - `entities`: Some components need access to the entities that was created as part of the
    ///               full prefab, for linking purposes, so this contains all those `Entity`s.
    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), SpecsError>;
}

/// Prefab data container
///
/// Contains a list prefab data for the entities affected by the prefab. The first entry in the
/// `entities` list will be applied to the main `Entity` the `Handle` is placed on, and all other
/// entries will trigger creation of a new entity. Note that the parent index is ignored for the
/// first entry in the list.
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
///         PrefabEntity { parent: 0 /* not used */, .. },
///         PrefabEntity { parent: 0, .. },
///         PrefabEntity { parent: 0, .. },
///         PrefabEntity { parent: 2, .. },
///     ],
/// }
/// ```
///
/// ### Type parameters:
///
/// - `T`: `PrefabData`
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Prefab<T> {
    entities: Vec<PrefabEntity<T>>,
}

/// Prefab for a single entity
///
/// ### Type parameters:
///
/// - `T`: `PrefabData`
#[derive(Debug, Deserialize, Serialize)]
pub struct PrefabEntity<T> {
    parent: usize,
    data: Option<T>,
}

impl<T> Default for PrefabEntity<T> {
    fn default() -> Self {
        PrefabEntity::new(0, None)
    }
}

impl<T> PrefabEntity<T> {
    /// New prefab entity
    pub fn new(parent: usize, data: Option<T>) -> Self {
        PrefabEntity { parent, data }
    }
}

impl<T> Prefab<T> {
    /// Create new empty prefab
    pub fn new() -> Self {
        Prefab {
            entities: vec![PrefabEntity::default()],
        }
    }

    /// Create a prefab with data for only the main `Entity`
    pub fn new_main(data: T) -> Self {
        Prefab {
            entities: vec![PrefabEntity::new(0, Some(data))],
        }
    }

    /// Set main `Entity` data
    pub fn main(&mut self, data: Option<T>) {
        self.entities[0].data = data;
    }

    /// Add a new entity to the prefab, with optional data.
    ///
    /// If parent is None, parent will be the main entity.
    pub fn add(&mut self, data: Option<T>, parent: Option<usize>) -> usize {
        let index = self.entities.len();
        self.entities
            .push(PrefabEntity::new(parent.unwrap_or(0), data));
        index
    }

    /// Create a new entity in the prefab, with no data
    pub fn new_entity(&mut self) -> usize {
        self.add(None, None)
    }
}

impl<T> Asset for Prefab<T>
where
    T: Send + Sync + 'static,
{
    const NAME: &'static str = "PREFAB";
    type Data = Self;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl<T> Into<Result<Prefab<T>, Error>> for Prefab<T> {
    fn into(self) -> Result<Prefab<T>, Error> {
        Ok(self)
    }
}

/// Convenience `PrefabData` for loading assets of type `A` using `Format` `F`.
///
/// ### Type parameters:
///
/// - `A`: `Asset`,
/// - `F`: `Format` for loading `A`
pub struct AssetPrefabData<A, F>
where
    A: Asset,
    F: Format<A>,
{
    /// Name of the asset to load
    pub name: String,
    /// Format of the asset to load
    pub format: F,
    /// Options of the asset to load
    pub options: F::Options,
}

impl<'a, A, F> PrefabData<'a> for AssetPrefabData<A, F>
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

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), SpecsError> {
        let handle = system_data.0.load(
            self.name.as_ref(),
            self.format.clone(),
            self.options.clone(),
            (),
            &system_data.2,
        );
        system_data.1.insert(entity, handle).map(|_| ())
    }
}

/// Helper structure for loading prefabs
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
    use super::*;
    use Loader;
    use amethyst_core::specs::{RunNow, World};
    use amethyst_core::{GlobalTransform, Time, Transform, TransformPrefabData};
    use rayon::ThreadPoolBuilder;
    use std::sync::Arc;

    type MyPrefab = TransformPrefabData;

    #[test]
    fn test_prefab_load() {
        let mut world = World::new();
        let pool = Arc::new(ThreadPoolBuilder::default().build().unwrap());
        world.add_resource(pool.clone());
        world.add_resource(Loader::new(".", pool));
        world.add_resource(Time::default());
        let mut system = PrefabLoaderSystem::<MyPrefab>::default();
        RunNow::setup(&mut system, &mut world.res);

        let prefab = Prefab::new_main(TransformPrefabData::default());

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
        assert!(
            world
                .read_storage::<GlobalTransform>()
                .get(root_entity)
                .is_some()
        );
    }
}
