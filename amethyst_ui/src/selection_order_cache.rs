use amethyst_core::specs::{
    storage::GenericReadStorage, Entities, Entity, Join, ReadStorage, System, Write,
};
use derive_new::new;
use hibitset::BitSet;
use std::{cmp::Ordering, marker::PhantomData};

use crate::{Selectable, Selected};

// TODO: Optimize by using a tree. Should we enforce tab order = unique? Sort on insert.
/// A cache sorted by tab order and then by Entity.
/// Used to quickly find the next or previous selectable entities.
#[derive(Debug, Clone, Default)]
pub struct CachedSelectionOrder {
    /// The cached bitset.
    pub cached: BitSet,
    /// The cache holding the selection order and the corresponding entity.
    pub cache: Vec<(u32, Entity)>,
}

impl CachedSelectionOrder {
    /// Returns the index of the highest cached element (index in the cache!) that is currently selected.
    pub fn highest_order_selected_index<T: GenericReadStorage<Component = Selected>>(
        &self,
        selected_storage: &T,
    ) -> Option<usize> {
        self.cache
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (_, e))| selected_storage.get(*e).is_some())
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

/// System in charge of updating the CachedSelectionOrder resource on each frame.
#[derive(Debug, Default, new)]
pub struct CacheSelectionOrderSystem<G> {
    phantom: PhantomData<G>,
}

impl<'a, G> System<'a> for CacheSelectionOrderSystem<G>
where
    G: PartialEq + Send + Sync + 'static,
{
    type SystemData = (
        Entities<'a>,
        Write<'a, CachedSelectionOrder>,
        ReadStorage<'a, Selectable<G>>,
    );
    fn run(&mut self, (entities, mut cache, selectables): Self::SystemData) {
        {
            let mut rm = vec![];
            cache.cache.retain(|&(_t, entity)| {
                let keep = selectables.contains(entity);
                if !keep {
                    rm.push(entity.id());
                }
                keep
            });
            rm.iter().for_each(|e| {
                &mut cache.cached.remove(*e);
                ()
            });
        }

        for &mut (ref mut t, entity) in &mut cache.cache {
            *t = selectables.get(entity).unwrap().order;
        }

        // Attempt to insert the new entities in sorted position.  Should reduce work during
        // the sorting step.
        let transform_set = selectables.mask().clone();
        {
            let mut inserts = vec![];
            let mut pushes = vec![];
            {
                // Create a bitset containing only the new indices.
                let new = (&transform_set ^ &cache.cached) & &transform_set;
                for (entity, selectable, _new) in (&*entities, &selectables, &new).join() {
                    let pos = cache
                        .cache
                        .iter()
                        .position(|&(cached_t, _)| selectable.order < cached_t);
                    match pos {
                        Some(pos) => inserts.push((pos, (selectable.order, entity))),
                        None => pushes.push((selectable.order, entity)),
                    }
                }
            }
            inserts.iter().for_each(|e| cache.cache.insert(e.0, e.1));
            pushes.iter().for_each(|e| cache.cache.push(*e));
        }
        cache.cached = transform_set;

        // Sort from smallest tab order to largest tab order, then by entity creation time.
        // Most of the time this shouldn't do anything but you still need it for if the tab orders
        // change.
        cache
            .cache
            .sort_unstable_by(|&(t1, ref e1), &(t2, ref e2)| {
                let ret = t1.cmp(&t2);
                if ret == Ordering::Equal {
                    return e1.cmp(e2);
                }
                ret
            });
    }
}
