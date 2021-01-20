use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};

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
/// use amethyst::{core::Named, ecs::*};
///
/// let mut world = World::default();
/// let entity: Entity = world.push((Named("Reginald".into()),));
/// ```
///
/// Creating a name from a dynamically generated string:
///
/// ```
/// use amethyst::{core::Named, ecs::*};
///
/// let mut world = World::default();
/// for entity_num in 0..10 {
///     world.push((Named(format!("Entity Number {}", entity_num).into()),));
/// }
/// ```
///
/// Accessing a named entity in a system:
/// ```
/// use amethyst::{core::Named, ecs::*};
///
/// SystemBuilder::new("NamedSystem")
///     .with_query(<(Entity, Read<Named>)>::query())
///     .build(move |_commands, world, _resource, query| {
///         for (entity, name) in query.iter(world) {
///             println!("Entity {:?} is named {}", entity, name);
///         }
///     });
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Named(
    /// The name of the entity this component is attached to.
    pub Cow<'static, str>,
);

impl Display for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Named {
    /// Creates a new instance of `Named`
    pub fn new<T: Into<Cow<'static, str>>>(name: T) -> Self {
        Named(name.into())
    }
}
