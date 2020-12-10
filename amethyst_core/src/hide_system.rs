#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{ecs::*, transform::components::Parent, HiddenPropagate};

/// This system adds a [HiddenPropagate](struct.HiddenPropagate.html)-component to all children.
///
/// Using this system will result in every child being hidden.
#[derive(Debug)]
pub struct HideHierarchySystem;

impl HideHierarchySystem {
    /// Creates a new `HideHierarchySystem`.
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(&'static mut self) -> Box<dyn Runnable> {
        Box::new(
            SystemBuilder::<()>::new("UiTransformSystem")
                .with_query(
                    <(Entity, TryWrite<HiddenPropagate>, Read<Parent>)>::query()
                        .filter(maybe_changed::<HiddenPropagate>()),
                )
                .build(move |commands, world, _, query| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("hide_hierarchy_system");

                    // collect the hidden things
                    let propagates: Vec<(Entity, HiddenPropagate)> = {
                        <(Entity, Read<HiddenPropagate>)>::query()
                            .iter(world)
                            .map(|x| (*x.0, x.1.clone()))
                            .collect()
                    };

                    // attaches HiddenPropagate to children
                    for (entity, hidden, parent) in query.iter_mut(world) {
                        {
                            let self_is_manually_hidden =
                                hidden.map_or(false, |p| !p.is_propagated);

                            let hidden_parent = propagates.iter().find(|x| x.0 == parent.0);

                            if !self_is_manually_hidden {
                                if hidden_parent.is_some() {
                                    commands.add_component(
                                        *entity,
                                        HiddenPropagate {
                                            is_propagated: true,
                                        },
                                    );
                                } else {
                                    commands.remove_component::<HiddenPropagate>(*entity);
                                }
                            }
                        }
                    }
                }),
        )
    }
}
