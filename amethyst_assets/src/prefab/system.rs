use std::{marker::PhantomData, ops::Deref};

use log::error;

use amethyst_core::{
    specs::{
        storage::ComponentEvent, BitSet, Entities, Entity, Join, Read, ReadExpect, ReadStorage,
        ReaderId, Resources, System, Write, WriteStorage,
    },
    ArcThreadPool, Parent, Time,
};

use crate::{AssetStorage, Completion, Handle, HotReloadStrategy, ProcessingState, ResultExt};

use super::{Prefab, PrefabData, PrefabTag};

/// System that load `Prefab`s for `PrefabData` `T`.
///
/// ### Type parameters:
///
/// - `T`: `PrefabData`
pub struct PrefabLoaderSystem<T> {
    _m: PhantomData<T>,
    entities: Vec<Entity>,
    finished: Vec<Entity>,
    to_process: BitSet,
    insert_reader: Option<ReaderId<ComponentEvent>>,
    next_tag: u64,
}

impl<T> Default for PrefabLoaderSystem<T> {
    fn default() -> Self {
        PrefabLoaderSystem {
            _m: PhantomData,
            entities: Vec::default(),
            finished: Vec::default(),
            to_process: BitSet::default(),
            insert_reader: None,
            next_tag: 0,
        }
    }
}

impl<'a, T> System<'a> for PrefabLoaderSystem<T>
where
    T: PrefabData<'a> + Send + Sync + 'static,
{
    type SystemData = (
        Entities<'a>,
        Write<'a, AssetStorage<Prefab<T>>>,
        ReadStorage<'a, Handle<Prefab<T>>>,
        Read<'a, Time>,
        ReadExpect<'a, ArcThreadPool>,
        Option<Read<'a, HotReloadStrategy>>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, PrefabTag<T>>,
        T::SystemData,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut prefab_storage,
            prefab_handles,
            time,
            pool,
            strategy,
            mut parents,
            mut tags,
            mut prefab_system_data,
        ) = data;
        let strategy = strategy.as_ref().map(Deref::deref);
        prefab_storage.process(
            |mut d| {
                d.tag = Some(self.next_tag);
                self.next_tag += 1;
                if !d.loading() {
                    if !d
                        .load_sub_assets(&mut prefab_system_data)
                        .chain_err(|| "Failed starting sub asset loading")?
                    {
                        return Ok(ProcessingState::Loaded(d));
                    }
                }
                match d.progress().complete() {
                    Completion::Complete => Ok(ProcessingState::Loaded(d)),
                    Completion::Failed => {
                        error!("Failed loading sub asset: {:?}", d.progress().errors());
                        Err("Failed loading sub asset")?
                    }
                    Completion::Loading => Ok(ProcessingState::Loading(d)),
                }
            },
            time.frame_number(),
            &**pool,
            strategy,
        );
        prefab_handles
            .channel()
            .read(self.insert_reader.as_mut().expect(
                "`PrefabLoaderSystem::setup` was not called before `PrefabLoaderSystem::run`",
            ))
            .for_each(|event| {
                if let ComponentEvent::Inserted(id) = event {
                    self.to_process.add(*id);
                }
            });
        self.finished.clear();
        for (root_entity, handle, _) in (&*entities, &prefab_handles, &self.to_process).join() {
            if let Some(prefab) = prefab_storage.get(handle) {
                self.finished.push(root_entity);
                // create entities
                self.entities.clear();
                self.entities.push(root_entity);
                for entity_data in prefab.entities.iter().skip(1) {
                    let new_entity = entities.create();
                    self.entities.push(new_entity);
                    if let Some(parent) = entity_data.parent {
                        parents
                            .insert(
                                new_entity,
                                Parent {
                                    entity: self.entities[parent],
                                },
                            )
                            .expect("Unable to insert `Parent` for prefab");
                    }
                    tags.insert(
                        new_entity,
                        PrefabTag::new(
                            prefab.tag.expect(
                                "Unreachable: Every loaded prefab should have a `PrefabTag`",
                            ),
                        ),
                    )
                    .expect("Unable to insert `PrefabTag` for prefab entity");
                }
                // create components
                for (index, entity_data) in prefab.entities.iter().enumerate() {
                    if let Some(ref prefab_data) = &entity_data.data {
                        prefab_data
                            .add_to_entity(
                                self.entities[index],
                                &mut prefab_system_data,
                                &self.entities,
                            )
                            .expect("Unable to add prefab system data to entity");
                    }
                }
            }
        }

        for entity in &self.finished {
            self.to_process.remove(entity.id());
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.insert_reader = Some(WriteStorage::<Handle<Prefab<T>>>::fetch(&res).register_reader());
    }
}
