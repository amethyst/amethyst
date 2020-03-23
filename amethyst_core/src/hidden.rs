use crate::ecs::{
    prelude::Component,
    storage::{DenseVecStorage, FlaggedStorage, NullStorage},
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
#[derive(Clone, Debug)]
pub struct HiddenPropagate {
    /// Whether this is inserted automatically by propagation though the `ParentHierarchy`.
    ///
    /// If true, then the `HideHierarchySystem` should manage (insert / remove) the component.
    /// If the user inserts it themselves, then the `HideHierarchySystem` should not remove it.
    pub(crate) is_propagated: bool,
}

impl Component for HiddenPropagate {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl HiddenPropagate {
    /// Creates an instance of HiddenPropagate.
    pub fn new() -> Self {
        Self {
            is_propagated: false,
        }
    }

    /// Is meant to be used only by HideHierarchySystem.
    pub(crate) fn new_propagated() -> Self {
        Self {
            is_propagated: true,
        }
    }

    /// Returns true if this component was propagated by [HideHierarchySystem](struct.HideHierarchySystem.html) automatically.
    pub fn is_propagated(&self) -> bool {
        self.is_propagated
    }
}
