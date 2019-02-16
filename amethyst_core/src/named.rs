use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use specs::{world::LazyBuilder, Component, DenseVecStorage, EntityBuilder, WriteStorage};

/// A component that gives a name to an [`Entity`].
///
/// There are two ways you can get a name for an entity:
///
/// * Hard-coding the entity name in code, in which case the name would be a [`&'static str`][str].
/// * Dynamically generating the string or loading it from a data file, in which case the name
///   would be a `String`.
///
/// To support both of these cases smoothly, `Named` stores the name as [`Cow<'static, str>`].
/// You can pass either a [`&'static str`][str] or a [`String`] to [`Named::new`], and your code
/// can generally treat the `name` field as a [`&str`][str] without needing to know whether the
/// name is actually an owned or borrowed string.
///
/// [`Entity`]: https://docs.rs/specs/*/specs/struct.Entity.html
/// [`Cow<'static, str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [str]: https://doc.rust-lang.org/std/primitive.str.html
/// [`Named::new`]: #method.new
///
/// # Examples
///
/// Creating a name from string constant:
///
/// ```
/// use amethyst::core::{Named, WithNamed};
/// use amethyst::ecs::prelude::*;
///
/// let mut world = World::new();
/// world.register::<Named>();
///
/// world
///     .create_entity()
///     .named("Super Cool Entity")
///     .build();
/// ```
///
/// Creating a name from a dynamically generated string:
///
/// ```
/// use amethyst::core::{Named, WithNamed};
/// use amethyst::ecs::prelude::*;
///
/// let mut world = World::new();
/// world.register::<Named>();
///
/// for entity_num in 0..10 {
///     world
///         .create_entity()
///         .named(format!("Entity Number {}", entity_num))
///         .build();
/// }
/// ```
///
/// Accessing a named entity in a system:
///
/// ```
/// use amethyst::core::Named;
/// use amethyst::ecs::prelude::*;
///
/// pub struct NameSystem;
///
/// impl<'s> System<'s> for NameSystem {
///     type SystemData = (
///         Entities<'s>,
///         ReadStorage<'s, Named>,
///     );
///
///     fn run(&mut self, (entities, names): Self::SystemData) {
///         for (entity, name) in (&*entities, &names).join() {
///             println!("Entity {:?} is named {}", entity, name.name);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Named {
    /// The name of the entity this component is attached to.
    pub name: Cow<'static, str>,
}

impl Named {
    /// Constructs a new `Named` from a string.
    ///
    /// # Examples
    ///
    /// From a string constant:
    ///
    /// ```
    /// use amethyst::core::Named;
    ///
    /// let name_component = Named::new("Super Cool Entity");
    /// ```
    ///
    /// From a dynamic string:
    ///
    /// ```
    /// use amethyst::core::Named;
    ///
    /// let entity_num = 7;
    /// let name_component = Named::new(format!("Entity Number {}", entity_num));
    /// ```
    pub fn new<S>(name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Named { name: name.into() }
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
    fn named<S>(self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>;
}

impl<'a> WithNamed for EntityBuilder<'a> {
    fn named<S>(self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.world
            .system_data::<(WriteStorage<'a, Named>,)>()
            .0
            .insert(self.entity, Named::new(name))
            .expect("Unreachable: Entities should always be valid when just created");
        self
    }
}

impl<'a> WithNamed for LazyBuilder<'a> {
    fn named<S>(self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.lazy.insert::<Named>(self.entity, Named::new(name));
        self
    }
}
