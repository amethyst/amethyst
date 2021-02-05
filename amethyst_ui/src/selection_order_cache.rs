use std::{collections::HashSet, marker::PhantomData};

use amethyst_core::ecs::*;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{Selectable, Selected};

/// Resource dedicated to the CacheSelectionOrderSystem behaviour
#[derive(Debug, Clone, Default)]
pub struct CachedSelectionOrderResource {
    /// A track of current cached entities
    pub cached: HashSet<Entity>,
    /// The cache holding the selection order and the corresponding entity.
    pub cache: Vec<(u32, Entity)>,
}

impl CachedSelectionOrderResource {
    /// Returns the index of the highest cached element (index in the cache!) that is currently selected.
    pub fn highest_order_selected_index<'a, T>(&self, selected_storage: T) -> Option<usize>
    where
        T: Iterator<Item = (&'a Entity, &'a mut Selected)>,
    {
        let everything: Vec<&'a Entity> = selected_storage.into_iter().map(|(e, _)| e).collect();

        self.cache
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (_, e1))| everything.contains(&e1))
            .map(|t| t.0)
    }

    /// Returns the index in the cache for the specified entity.
    pub fn index_of(&self, entity: Entity) -> Option<usize> {
        self.cache
            .iter()
            .enumerate()
            .find(|(_, (_, e))| *e == entity)
            .map(|t| t.0)
    }
}

// TODO: Optimize by using a tree. Should we enforce tab order = unique? Sort on insert.
/// A cache sorted by tab order and then by Entity.
/// Used to quickly find the next or previous selectable entities.
#[derive(Debug)]
pub struct CacheSelectionSystem<G> {
    _m: PhantomData<G>,
}

impl<G> CacheSelectionSystem<G> {
    /// Constructs a new `CacheSelectionSystem<G>`.
    pub fn new() -> CacheSelectionSystem<G> {
        CacheSelectionSystem { _m: PhantomData }
    }
}

impl<G> System for CacheSelectionSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("CacheSelectionOrderSystem")
                .write_resource::<CachedSelectionOrderResource>()
                .with_query(<(Entity, &Selectable<G>)>::query())
                .build(move |_commands, world, cache, selectables| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("cache_selection_order_system");

                    {
                        let mut rm = vec![];
                        cache.cache.retain(|&(_t, entity)| {
                            let keep = selectables.get(world, entity).is_ok();
                            if !keep {
                                rm.push(entity);
                            }
                            keep
                        });
                        rm.iter().for_each(|e| {
                            cache.cached.remove(e);
                        });
                    }

                    for &mut (ref mut t, entity) in &mut cache.cache {
                        *t = selectables.get(world, entity).unwrap().1.order;
                    }

                    // ---------

                    // Attempt to insert the new entities in sorted position.  Should reduce work during
                    // the sorting step.
                    {
                        let mut inserts = vec![];
                        let mut pushes = vec![];
                        {
                            selectables.for_each(world, |(entity, selectable)| {
                                // We only want the new ones.
                                // The old way (pre legion) to do it was with bitset :
                                // let new = (&transform_set ^ &cache.cached) & &transform_set;
                                if !cache.cached.contains(entity) {
                                    let pos = cache
                                        .cache
                                        .iter()
                                        .position(|&(cached_t, _)| selectable.order < cached_t);

                                    match pos {
                                        Some(pos) => {
                                            inserts.push((pos, (selectable.order, entity)))
                                        }
                                        None => pushes.push((selectable.order, entity)),
                                    }
                                }
                            });
                        }
                        inserts
                            .iter()
                            .for_each(|(pos, t)| cache.cache.insert(*pos, (t.0, *t.1)));
                        pushes
                            .iter()
                            .for_each(|(order, t)| cache.cache.push((*order, **t)));
                    }
                    // Update the cached with all entities

                    cache.cached.clear();
                    selectables.for_each(world, |(entity, _)| {
                        cache.cached.insert(*entity);
                    });

                    cache
                        .cache
                        .sort_unstable_by(|&(t1, _), &(t2, _)| t1.cmp(&t2));
                }),
        )
    }
}
