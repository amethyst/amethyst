use amethyst_core::{
    specs::prelude::{
        BitSet, InsertedFlag, ReadExpect, ReadStorage, ReaderId, RemovedFlag, Resources, System,
        WriteStorage,
    },
    transform::components::{HierarchyEvent, Parent, ParentHierarchy},
};
use amethyst_renderer::Hidden;

// Based on the [UiTransformSystem](struct.UiTransformSystem.html).
/// This system adds a [Hidden](struct.Hidden.html)-component to all children.
/// Using this system will result in every child being hidden, as it is currently not possible to make individual choices.
#[derive(Default)]
pub struct HideHierarchySystem {
    hidden_entities: BitSet,

    inserted_hidden_id: Option<ReaderId<InsertedFlag>>,
    removed_hidden_id: Option<ReaderId<RemovedFlag>>,

    parent_events_id: Option<ReaderId<HierarchyEvent>>,
}

impl<'a> System<'a> for HideHierarchySystem {
    type SystemData = (
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Parent>,
        ReadExpect<'a, ParentHierarchy>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut hidden, parents, hierarchy) = data;
        #[cfg(feature = "profiler")]
        profile_scope!("hide_hierarchy_system");

        self.hidden_entities.clear();

        hidden.populate_inserted(
            &mut self.inserted_hidden_id.as_mut().unwrap(),
            &mut self.hidden_entities,
        );
        hidden.populate_removed(
            &mut self.removed_hidden_id.as_mut().unwrap(),
            &mut self.hidden_entities,
        );

        for event in hierarchy
            .changed()
            .read(&mut self.parent_events_id.as_mut().unwrap())
        {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    self.hidden_entities.add(entity.id());
                }
                HierarchyEvent::Modified(entity) => {
                    self.hidden_entities.add(entity.id());
                }
                //_ => {error!("Non-implemented HierarchyEvent received.");},
            }
        }

        // Compute hidden with parents.
        for entity in hierarchy.all() {
            {
                let self_dirty = self.hidden_entities.contains(entity.id());
                let parent_entity = parents.get(*entity).unwrap().entity;
                let parent_dirty = self.hidden_entities.contains(parent_entity.id());
                if parent_dirty {
                    if hidden.contains(parent_entity) {
                        for child in hierarchy.children(parent_entity) {
                            match hidden.insert(*child, Hidden::default()) {
                                Ok(_v) => (),
                                Err(e) => {
                                    error!(
                                        "Failed to add Hidden component to child-entity {:?}. {:?}",
                                        *entity, e
                                    );
                                }
                            }
                        }
                    } else {
                        for child in hierarchy.children(parent_entity) {
                            hidden.remove(*child);
                        }
                    }
                } else if self_dirty {
                    if hidden.contains(*entity) {
                        for child in hierarchy.children(*entity) {
                            match hidden.insert(*child, Hidden::default()) {
                                Ok(_v) => (),
                                Err(e) => {
                                    error!(
                                        "Failed to add Hidden component to child-entity {:?}. {:?}",
                                        *entity, e
                                    );
                                }
                            }
                        }
                    } else {
                        for child in hierarchy.children(*entity) {
                            hidden.remove(*child);
                        }
                    }
                }
            }
            // Populate the modifications we just did.
            hidden.populate_inserted(
                &mut self.inserted_hidden_id.as_mut().unwrap(),
                &mut self.hidden_entities,
            );
            hidden.populate_removed(
                &mut self.removed_hidden_id.as_mut().unwrap(),
                &mut self.hidden_entities,
            );
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.parent_events_id = Some(res.fetch_mut::<ParentHierarchy>().track());
        let mut hidden = WriteStorage::<Hidden>::fetch(res);
        self.inserted_hidden_id = Some(hidden.track_inserted());
        self.removed_hidden_id = Some(hidden.track_removed());
    }
}
