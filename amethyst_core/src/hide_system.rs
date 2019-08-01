use crate::{
    ecs::prelude::{
        BitSet, ComponentEvent, ReadExpect, ReadStorage, ReaderId, System, SystemData, World,
        WriteStorage,
    },
    transform::components::{HierarchyEvent, Parent, ParentHierarchy},
    SystemDesc,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use log::error;

use crate::HiddenPropagate;

/// Builds a `HideHierarchySystem`.
#[derive(Default, Debug)]
pub struct HideHierarchySystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, HideHierarchySystem> for HideHierarchySystemDesc {
    fn build(self, world: &mut World) -> HideHierarchySystem {
        <HideHierarchySystem as System<'_>>::SystemData::setup(world);

        // This fetch_mut panics if `ParentHierarchy` is not set up yet, hence the dependency on
        // "parent_hierarchy_system"
        let parent_events_id = world.fetch_mut::<ParentHierarchy>().track();
        let mut hidden = WriteStorage::<HiddenPropagate>::fetch(&world);
        let hidden_events_id = hidden.register_reader();

        HideHierarchySystem::new(hidden_events_id, parent_events_id)
    }
}

/// This system adds a [HiddenPropagate](struct.HiddenPropagate.html)-component to all children.
///
/// Using this system will result in every child being hidden.
/// Depends on the resource "ParentHierarchy", which is set up by the
/// [TransformBundle](struct.TransformBundle.html)
///
/// Based on the [UiTransformSystem](struct.UiTransformSystem.html).
#[derive(Debug)]
pub struct HideHierarchySystem {
    marked_as_modified: BitSet,
    hidden_events_id: ReaderId<ComponentEvent>,
    parent_events_id: ReaderId<HierarchyEvent>,
}

impl HideHierarchySystem {
    /// Creates a new `HideHierarchySystem`.
    pub fn new(
        hidden_events_id: ReaderId<ComponentEvent>,
        parent_events_id: ReaderId<HierarchyEvent>,
    ) -> Self {
        Self {
            marked_as_modified: BitSet::default(),
            hidden_events_id,
            parent_events_id,
        }
    }
}

impl<'a> System<'a> for HideHierarchySystem {
    type SystemData = (
        WriteStorage<'a, HiddenPropagate>,
        ReadStorage<'a, Parent>,
        ReadExpect<'a, ParentHierarchy>,
    );
    fn run(&mut self, (mut hidden, parents, hierarchy): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("hide_hierarchy_system");

        self.marked_as_modified.clear();

        // Borrow multiple parts of self as mutable
        let self_hidden_events_id = &mut self.hidden_events_id;
        let self_marked_as_modified = &mut self.marked_as_modified;

        hidden
            .channel()
            .read(self_hidden_events_id)
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Removed(id) => {
                    self_marked_as_modified.add(*id);
                }
                ComponentEvent::Modified(_id) => {}
            });

        for event in hierarchy.changed().read(&mut self.parent_events_id) {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    self_marked_as_modified.add(entity.id());
                }
                HierarchyEvent::Modified(entity) => {
                    // HierarchyEvent::Modified includes insertion of new components to the storage.
                    self_marked_as_modified.add(entity.id());
                }
            }
        }

        // Compute hide status with parents.
        for entity in hierarchy.all() {
            {
                let self_dirty = self_marked_as_modified.contains(entity.id());

                let parent_entity = parents
                    .get(*entity)
                    .expect(
                        "Unreachable: All entities in `ParentHierarchy` should also be in \
                         `Parents`",
                    )
                    .entity;
                let parent_dirty = self_marked_as_modified.contains(parent_entity.id());
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
                    // in case the parent was already dirty, this entity and its children have
                    // already been hidden, therefore it only needs to be an else-if, instead of a
                    // stand-alone if.
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
            // Happens inside the for-loop, so that the changes are picked up in the next iteration
            // already, instead of on the next `system.run()`
            hidden
                .channel()
                .read(self_hidden_events_id)
                .for_each(|event| match event {
                    ComponentEvent::Inserted(id) | ComponentEvent::Removed(id) => {
                        self_marked_as_modified.add(*id);
                    }
                    ComponentEvent::Modified(_id) => {}
                });
        }
    }
}
