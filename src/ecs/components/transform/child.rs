extern crate specs;

use self::specs::{Component, VecStorage, Entity};
use std::sync::atomic::{AtomicBool, Ordering};

/// Component for defining a parent entity.
pub struct Child {
    /// The parent entity
    parent: Entity,

    /// Flag for whether the child was changed
    dirty: AtomicBool,
}

impl Child {
    pub fn new(entity: Entity) -> Child {
        Child {
            parent: entity,
            dirty: AtomicBool::new(true),
        }
    }

    #[inline]
    pub fn parent(&self) -> Entity {
        self.parent
    }
    #[inline]
    pub fn set_parent(&mut self, entity: Entity) {
        self.parent = entity;
        self.flag(true);
    }

    /// Flag that parent has been changed
    ///
    /// Note: `set_parent` flags the parent.
    #[inline]
    pub fn flag(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst);
    }

    /// Returns whether the parent was changed.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
}

impl Component for Child {
    type Storage = VecStorage<Child>;
}
