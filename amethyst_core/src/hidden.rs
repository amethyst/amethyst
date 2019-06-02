use crate::ecs::{
    prelude::Component,
    storage::{FlaggedStorage, NullStorage},
};

/// HiddenComponent mesh component
/// Useful for entities, that should not be rendered, but stay loaded in memory.
#[derive(Clone, Debug, Default)]
pub struct HiddenComponent;

impl Component for HiddenComponent {
    type Storage = NullStorage<Self>;
}

/// Like [HiddenComponent](struct.Hidden.html), but can propagate through children when the [HideHierarchySystem](struct.HideHierarchySystem.html)
/// is enabled in the [RenderBundle](struct.RenderBundle.html).
#[derive(Clone, Debug, Default)]
pub struct HiddenPropagateComponent;

impl Component for HiddenPropagateComponent {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}
