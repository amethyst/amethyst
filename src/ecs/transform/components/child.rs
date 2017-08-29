use std::sync::atomic::{AtomicBool, Ordering};

use ecs::{Component, VecStorage, Entity};

/// Component for defining a parent entity.
pub struct Child {
    /// The parent entity
    parent: Entity,
    /// Flags whether the child was changed
    dirty: AtomicBool,
}

impl Child {
    /// Creates a new child
    pub fn new(entity: Entity) -> Child {
        Child {
            parent: entity,
            dirty: AtomicBool::new(true),
        }
    }

    /// Returns our parent entity.
    #[inline]
    pub fn parent(&self) -> Entity {
        self.parent
    }

    /// Sets the given entity as our parent.
    #[inline]
    pub fn set_parent(&mut self, entity: Entity) {
        self.parent = entity;
        self.flag(true);
    }

    /// Signals to our parent entity that its child entity has changed.
    ///
    /// Note: Calling `set_parent()` flags the parent as dirty.
    #[inline]
    pub fn flag(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst);
    }

    /// Returns whether the parent entity has changed.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
}

impl Component for Child {
    type Storage = VecStorage<Child>;
}
