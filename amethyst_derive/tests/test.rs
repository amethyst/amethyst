#![allow(
    clippy::unneeded_field_pattern,
    clippy::block_in_if_condition_stmt,
    clippy::unneeded_field_pattern
)]
use amethyst_derive::{EventReader, PrefabData};

use amethyst_assets::{PrefabData, ProgressCounter};
use amethyst_core::{
    ecs::{Component, DenseVecStorage, Entity, Read, SystemData, World, WriteStorage},
    shrev::{EventChannel, ReaderId},
    EventReader,
};
use amethyst_error::Error;

#[derive(Clone)]
pub struct TestEvent1;

#[derive(Clone)]
pub struct TestEvent2;

#[derive(Clone)]
pub struct TestEvent3<T>(T);

#[derive(Clone, EventReader)]
#[reader(TestEventReader)]
pub enum TestEvent {
    One(TestEvent1),
    Two(TestEvent2),
}

#[derive(Clone, EventReader)]
#[reader(TestEventWithTypeParameterReader)]
pub enum TestEventWithTypeParameter<T1, T2>
where
    T1: Clone + Send + Sync + 'static,
    T2: Clone + Send + Sync + 'static,
{
    One(TestEvent1),
    Two(TestEvent2),
    Three(TestEvent3<T1>),
    Four(TestEvent3<T2>),
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

#[derive(PrefabData, Clone)]
pub struct OuterTuple(#[prefab(Component)] External);

#[derive(PrefabData, Clone)]
pub enum EnumPrefab {
    One {
        number: Stuff<usize>,
    },
    Two {
        #[prefab(Component)]
        component: External,
    },
    Three {},
    Four,
    Five(Stuff<String>, #[prefab(Component)] External),
}

#[cfg(test)]
mod tests {
    use super::*;
    use amethyst_assets::{AssetStorage, Loader, Prefab, PrefabLoaderSystemDesc};
    use amethyst_core::ecs::{world::EntitiesRes, Builder, Join, WorldExt};
    use amethyst_test::prelude::*;

    macro_rules! assert_prefab {
        ($prefab_type:ident, $prefab:expr, $assertion:expr) => {
            assert!(AmethystApplication::blank()
                .with_system_desc(
                    PrefabLoaderSystemDesc::<$prefab_type>::default(),
                    "test_loader",
                    &[]
                )
                .with_effect(|world| {
                    let handle = {
                        let loader = world.read_resource::<Loader>();
                        let storage = world.read_resource::<AssetStorage<Prefab<$prefab_type>>>();
                        let mut prefab = Prefab::new();
                        prefab.main(Some($prefab));
                        loader.load_from_data(prefab, (), &storage)
                    };
                    world.create_entity().with(handle).build();
                })
                .with_assertion($assertion)
                .run()
                .is_ok())
        };
    }

    #[test]
    fn instantiate_struct_prefabs() {
        assert_prefab!(
            Outer,
            Outer {
                external: External { inner: 100 }
            },
            |world| {
                let entities = world.read_resource::<EntitiesRes>();
                let storage = world.read_storage::<External>();

                let entities_components = (&entities, &storage).join().collect::<Vec<_>>();

                assert_eq!(entities_components.len(), 1);
                entities_components
                    .into_iter()
                    .for_each(|(_, ex)| assert_eq!(ex.inner, 100));
            }
        );
    }

    #[test]
    fn instantiate_struct_variant() {
        assert_prefab!(
            EnumPrefab,
            EnumPrefab::One {
                number: Stuff { inner: 1 }
            },
            |world| {
                let entities = world.read_resource::<EntitiesRes>();
                let storage = world.read_storage::<Stuff<usize>>();

                let entities_components = (&entities, &storage).join().collect::<Vec<_>>();

                assert_eq!(entities_components.len(), 1);
                entities_components
                    .into_iter()
                    .for_each(|(_, ex)| assert_eq!(ex.inner, 1));
            }
        );
    }

    #[test]
    fn instantiate_struct_variant_with_component_field() {
        assert_prefab!(
            EnumPrefab,
            EnumPrefab::Two {
                component: External { inner: 2 }
            },
            |world| {
                let entities = world.read_resource::<EntitiesRes>();
                let storage = world.read_storage::<External>();

                let entities_components = (&entities, &storage).join().collect::<Vec<_>>();

                assert_eq!(entities_components.len(), 1);
                entities_components
                    .into_iter()
                    .for_each(|(_, ex)| assert_eq!(ex.inner, 2));
            }
        );
    }

    #[test]
    fn instantiate_tuple_variant() {
        assert_prefab!(
            EnumPrefab,
            EnumPrefab::Five(
                Stuff {
                    inner: "three".to_string()
                },
                External { inner: 4 }
            ),
            |world| {
                let entities = world.read_resource::<EntitiesRes>();
                let stuff_storage = world.read_storage::<Stuff<String>>();
                let external_storage = world.read_storage::<External>();

                let entities_components = (&entities, &stuff_storage, &external_storage)
                    .join()
                    .collect::<Vec<_>>();

                assert_eq!(entities_components.len(), 1);
                entities_components.into_iter().for_each(|(_, st, ex)| {
                    assert_eq!(st.inner, "three");
                    assert_eq!(ex.inner, 4);
                });
            }
        );
    }
}
