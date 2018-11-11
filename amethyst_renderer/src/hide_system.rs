use amethyst_core::{
    specs::prelude::{
        BitSet, InsertedFlag, ReadExpect, ReadStorage, ReaderId, RemovedFlag, Resources, System,
        WriteStorage,
    },
    transform::components::{HierarchyEvent, Parent, ParentHierarchy},
};

use HiddenPropagate;

// Based on the [UiTransformSystem](struct.UiTransformSystem.html).
/// This system adds a [HiddenPropagate](struct.HiddenPropagate.html)-component to all children.
/// Using this system will result in every child being hidden.
/// Depends on the resource "ParentHierarchy", which is set up by the [TransformBundle](struct.TransformBundle.html)
#[derive(Default)]
pub struct HideHierarchySystem {
    marked_as_modified: BitSet,

    inserted_hidden_id: Option<ReaderId<InsertedFlag>>,
    removed_hidden_id: Option<ReaderId<RemovedFlag>>,

    parent_events_id: Option<ReaderId<HierarchyEvent>>,
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

        let self_inserted_hidden_id = &mut self.inserted_hidden_id.as_mut().expect(
            "`HideHierarchySystem::setup` was not called before `HideHierarchySystem::run`",
        );
        let self_removed_hidden_id = &mut self.removed_hidden_id.as_mut().expect(
            "`HideHierarchySystem::setup` was not called before `HideHierarchySystem::run`",
        );

        hidden.populate_inserted(self_inserted_hidden_id, &mut self.marked_as_modified);
        hidden.populate_removed(self_removed_hidden_id, &mut self.marked_as_modified);

        for event in hierarchy
            .changed()
            .read(&mut self.parent_events_id.as_mut().expect(
                "`HideHierarchySystem::setup` was not called before `HideHierarchySystem::run`",
            )) {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    self.marked_as_modified.add(entity.id());
                }
                HierarchyEvent::Modified(entity) => {
                    // HierarchyEvent::Modified includes insertion of new components to the storage.
                    self.marked_as_modified.add(entity.id());
                }
            }
        }

        // Compute hide status with parents.
        for entity in hierarchy.all() {
            {
                let self_dirty = self.marked_as_modified.contains(entity.id());

                let parent_entity = parents.get(*entity).expect("Unreachable: All entities in `ParentHierarchy` should also be in `Parents`").entity;
                let parent_dirty = self.marked_as_modified.contains(parent_entity.id());
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
            hidden.populate_inserted(self_inserted_hidden_id, &mut self.marked_as_modified);
            hidden.populate_removed(self_removed_hidden_id, &mut self.marked_as_modified);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        // This fetch_mut panics if `ParentHierarchy` is not set up yet, hence the dependency on "parent_hierarchy_system"
        self.parent_events_id = Some(res.fetch_mut::<ParentHierarchy>().track());
        let mut hidden = WriteStorage::<HiddenPropagate>::fetch(res);
        self.inserted_hidden_id = Some(hidden.track_inserted());
        self.removed_hidden_id = Some(hidden.track_removed());
    }
}
