use amethyst_core::{
    specs::prelude::{
        BitSet, ComponentEvent, ReadExpect, ReadStorage, ReaderId, Resources, System, WriteExpect,
        WriteStorage,
    },
    transform::components::{HierarchyEvent, Parent, ParentHierarchy},
};

use crate::HiddenPropagate;

// Based on the [UiTransformSystem](struct.UiTransformSystem.html).
/// This system adds a [HiddenPropagate](struct.HiddenPropagate.html)-component to all children.
/// Using this system will result in every child being hidden.
/// Depends on the resource "ParentHierarchy", which is set up by the [TransformBundle](struct.TransformBundle.html)
#[derive(Default)]
pub struct HideHierarchySystem;

/// A resource for `HideHierarchySystem` which is automatically created and managed by
/// `HideHierarchySystem`.
pub struct HideHierarchySystemData {
    marked_as_modified: BitSet,
    hidden_events_id: ReaderId<ComponentEvent>,
    parent_events_id: ReaderId<HierarchyEvent>,
}

impl<'a> System<'a> for HideHierarchySystem {
    type SystemData = (
        WriteStorage<'a, HiddenPropagate>,
        ReadStorage<'a, Parent>,
        ReadExpect<'a, ParentHierarchy>,
        WriteExpect<'a, HideHierarchySystemData>,
    );
    fn run(&mut self, (mut hidden, parents, hierarchy, mut data): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("hide_hierarchy_system");

        data.marked_as_modified.clear();

        // Borrow multiple parts of self as mutable

        hidden
            .channel()
            .read(&mut data.hidden_events_id)
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Removed(id) => {
                    data.marked_as_modified.add(*id);
                }
                ComponentEvent::Modified(_id) => {}
            });

        for event in hierarchy.changed().read(&mut data.parent_events_id) {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    data.marked_as_modified.add(entity.id());
                }
                HierarchyEvent::Modified(entity) => {
                    // HierarchyEvent::Modified includes insertion of new components to the storage.
                    data.marked_as_modified.add(entity.id());
                }
            }
        }

        // Compute hide status with parents.
        for entity in hierarchy.all() {
            {
                let self_dirty = data.marked_as_modified.contains(entity.id());

                let parent_entity = parents.get(*entity).expect("Unreachable: All entities in `ParentHierarchy` should also be in `Parents`").entity;
                let parent_dirty = data.marked_as_modified.contains(parent_entity.id());
                if parent_dirty {
                    if hidden.contains(parent_entity) {
                        for child in hierarchy.all_children_iter(parent_entity) {
                            if let Err(e) = hidden.insert(child, HiddenPropagate::default()) {
                                error!("Failed to automatically add `HiddenPropagate`: {:?}", e);
                            };
                        }
                    } else {
                        for child in hierarchy.all_children_iter(parent_entity) {
                            hidden.remove(child);
                        }
                    }
                } else if self_dirty {
                    // in case the parent was already dirty, this entity and its children have already been hidden,
                    // therefore it only needs to be an else-if, instead of a stand-alone if.
                    if hidden.contains(*entity) {
                        for child in hierarchy.all_children_iter(*entity) {
                            if let Err(e) = hidden.insert(child, HiddenPropagate::default()) {
                                error!("Failed to automatically add `HiddenPropagate`: {:?}", e);
                            };
                        }
                    } else {
                        for child in hierarchy.all_children_iter(*entity) {
                            hidden.remove(child);
                        }
                    }
                }
            }
            // Populate the modifications we just did.
            // Happens inside the for-loop, so that the changes are picked up in the next iteration already,
            // instead of on the next `system.run()`
            hidden
                .channel()
                .read(&mut data.hidden_events_id)
                .for_each(|event| match event {
                    ComponentEvent::Inserted(id) | ComponentEvent::Removed(id) => {
                        data.marked_as_modified.add(*id);
                    }
                    ComponentEvent::Modified(_id) => {}
                });
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        // This fetch_mut panics if `ParentHierarchy` is not set up yet, hence the dependency on "parent_hierarchy_system"
        let parent_events_id = res.fetch_mut::<ParentHierarchy>().track();
        let hidden_events_id = WriteStorage::<HiddenPropagate>::fetch(res).register_reader();
        res.insert(HideHierarchySystemData {
            marked_as_modified: BitSet::new(),
            parent_events_id,
            hidden_events_id,
        });
    }
}
