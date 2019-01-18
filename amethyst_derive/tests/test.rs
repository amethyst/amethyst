use amethyst_derive::{EventReader, PrefabData};

use amethyst_assets::{PrefabData, PrefabError, ProgressCounter};
use amethyst_core::{
    shrev::{EventChannel, ReaderId},
    specs::{Component, DenseVecStorage, Entity, Read, Resources, SystemData, WriteStorage},
    EventReader,
};

#[derive(Clone)]
pub struct TestEvent1;

#[derive(Clone)]
pub struct TestEvent2;

#[derive(Clone, EventReader)]
#[reader(TestEventReader)]
pub enum TestEvent {
    One(TestEvent1),
    Two(TestEvent2),
}

#[derive(Clone, PrefabData, Default)]
#[prefab(Component)]
pub struct Stuff<T>
where
    T: Default + Clone + Send + Sync + 'static,
{
    inner: T,
}

impl<T> Component for Stuff<T>
where
    T: Clone + Default + Send + Sync + 'static,
{
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, PrefabData)]
pub struct OuterPrefab<T>
where
    T: Default + Clone + Send + Sync + 'static,
{
    inner: Stuff<T>,
}

#[derive(Clone)]
pub struct External {
    inner: u64,
}

impl Component for External {
    type Storage = DenseVecStorage<Self>;
}

#[derive(PrefabData, Clone)]
pub struct Outer {
    #[prefab(Component)]
    external: External,
}
