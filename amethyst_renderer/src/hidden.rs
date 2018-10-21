use amethyst_core::specs::{
    prelude::Component,
    storage::{FlaggedStorage, NullStorage},
};

/// Hidden mesh component
/// Useful for entities, that should not be rendered, but stay loaded in memory.
#[derive(Clone, Debug, Default)]
pub struct Hidden;

impl Component for Hidden {
    type Storage = NullStorage<Self>;
}

/// Like [Hidden](struct.Hidden.html), but can propagate through children when the [HideHierarchySystem](struct.HideHierarchySystem.html)
/// is enabled in the [RenderBundle](struct.RenderBundle.html).
#[derive(Clone, Debug, Default)]
pub struct HiddenPropagate;

impl Component for HiddenPropagate {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}
