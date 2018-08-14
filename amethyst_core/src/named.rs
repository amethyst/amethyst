use specs::{Component, DenseVecStorage, EntityBuilder, WriteStorage};

/// A component that gives a name to an `Entity`
pub struct Named {
    pub name: &'static str,
}

impl Named {
    pub fn new(name: &'static str) -> Self {
        Named { name }
    }
}

impl Component for Named {
    type Storage = DenseVecStorage<Self>;
}

/// An easy way to name an `Entity` and give it a `Named` `Component`.
pub trait WithNamed
where
    Self: Sized,
{
    /// Adds a name to the entity being built.
    fn named(self, name: &'static str) -> Self;
}

impl<'a> WithNamed for EntityBuilder<'a> {
    fn named(self, name: &'static str) -> Self {
        // Unwrap: The only way this can fail is if the entity is invalid and this is used while creating the entity.
        self.world
            .system_data::<(WriteStorage<'a, Named>,)>()
            .0
            .insert(self.entity, Named::new(name))
            .unwrap();
        self
    }
}
