use std::marker::PhantomData;
use std::ops::Deref;

use amethyst_core::specs::{Entities, Entity, Join, Read, ReadExpect, System, Write, WriteStorage};
use amethyst_core::{Parent, ThreadPool, Time};

use super::{Prefab, PrefabData};
use {AssetStorage, Handle, HotReloadStrategy};

/// System that load prefabs for `PrefabData` `T`.
///
/// ### Type parameters:
///
/// - `T`: `PrefabData`
#[derive(Default)]
pub struct PrefabLoaderSystem<T> {
    _m: PhantomData<T>,
    entities: Vec<Entity>,
    remove: Vec<Entity>,
}

impl<'a, T> System<'a> for PrefabLoaderSystem<T>
where
    T: PrefabData<'a> + Send + Sync + 'static,
{
    type SystemData = (
        Entities<'a>,
        Write<'a, AssetStorage<Prefab<T>>>,
        WriteStorage<'a, Handle<Prefab<T>>>,
        Read<'a, Time>,
        ReadExpect<'a, ThreadPool>,
        Option<Read<'a, HotReloadStrategy>>,
        WriteStorage<'a, Parent>,
        T::SystemData,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut prefab_storage,
            mut prefab_handles,
            time,
            pool,
            strategy,
            mut parents,
            mut prefab_system_data,
        ) = data;
        let strategy = strategy.as_ref().map(Deref::deref);
        prefab_storage.process(Into::into, time.frame_number(), &**pool, strategy);
        self.remove.clear();
        for (root_entity, handle) in (&*entities, &prefab_handles).join() {
            if let Some(prefab) = prefab_storage.get(handle) {
                self.remove.push(root_entity);
                // create entities
                self.entities.clear();
                self.entities.push(root_entity);
                for entity_data in prefab.entities.iter().skip(1) {
                    let new_entity = entities.create();
                    self.entities.push(new_entity);
                    parents
                        .insert(
                            new_entity,
                            Parent {
                                entity: self.entities[entity_data.parent],
                            },
                        )
                        .unwrap();
                }
                // create components
                for (index, entity_data) in prefab.entities.iter().enumerate() {
                    if let Some(ref prefab_data) = &entity_data.data {
                        prefab_data
                            .load_prefab(
                                self.entities[index],
                                &mut prefab_system_data,
                                &self.entities,
                            )
                            .unwrap();
                    }
                }
            }
        }

        for entity in &self.remove {
            prefab_handles.remove(*entity);
        }
    }
}
