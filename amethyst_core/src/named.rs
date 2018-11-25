use std::{borrow::Cow, ops::Deref};

use fnv::FnvHashMap as HashMap;
use specs::{
    storage::MaskedStorage, world::LazyBuilder, Component, DenseVecStorage, Entity, EntityBuilder,
    Storage, WriteStorage,
};

use util::{Cache, CachedStorage};

/// Extension trait to find an entity based on its `Named` component.
/// Can only be used on the `Named` component storage.
pub trait FindNamed {
    /// Finds an entity by its name by querying it from a map.
    ///
    /// For performance reasons, this should only be used once, and the returned `Entity` shall
    /// be stored to avoid future lookups.
    ///
    /// # Conflict behavior
    ///
    /// Only the last `Named` component with name `s` will be returned; if
    /// multiple entities with this name can exist, you can use `iter_with_name`
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use amethyst_core::{
    ///     specs::{Builder, EntityBuilder, ReadStorage, System, World},
    ///     FindNamed, Named, WithNamed,
    /// };
    ///
    /// let mut world = World::new();
    /// world.register::<Named>();
    ///
    /// let entity = world
    ///     .create_entity()
    ///     .named("My named Entity")
    ///     .build();
    ///
    /// // Get the `Named` storage from the `World`
    /// // This is equivalent to adding `ReadStorage<Named>` to your system.
    /// let named_storage = world.read_storage::<Named>();
    ///
    /// let search = named_storage.find_named("My named Entity");
    /// assert_eq!(search, Some(entity));
    ///
    /// let none = named_storage.find_named("Not existent");
    /// assert_eq!(none, None);
    /// ```
    fn find_named<S>(&self, s: S) -> Option<Entity>
    where
        S: AsRef<str>;

    /// Writes all entities where `matcher` returned `true` after being passed the name of that
    /// entity to the buffer.
    ///
    /// `buffer` does not get cleared automatically, if there are existing entities in the `Vec`
    /// it will just append the found entities.
    ///
    /// # Alternatives
    ///
    /// The method `find_named` can search for a single entity with an exact
    /// name, whereas this method allows arbitrary code to determine if an
    /// entity gets returned.
    ///
    /// If you immediately need access to a component of that entity, you might
    /// also consider to join over the components yourself:
    ///
    /// ```
    /// use amethyst_core::{specs::prelude::*, Named};
    ///
    /// pub struct NumChars {
    ///     /// Number of characters
    ///     num: usize,
    /// }
    ///
    /// impl Component for NumChars {
    ///     type Storage = VecStorage<Self>;
    /// }
    ///
    /// /// Writes the number of chars of all lowercase names to `NumChars` components.
    /// pub struct CountChars;
    ///
    /// impl<'a> System<'a> for CountChars {
    ///     type SystemData = (Entities<'a>, ReadStorage<'a, Named>, WriteStorage<'a, NumChars>);
    ///
    ///     fn run(&mut self, (entities, named, mut num_chars): Self::SystemData) {
    ///         let filter = |name| name.is_lowercase();
    ///
    ///         (&*entities, named)
    ///             .join()
    ///             .filter(|&(_, named)| filter(named.name()))
    ///             .for_each(|(entity, name)|
    ///                 num_chars.insert(entity, name.name().chars().count()
    ///                     .expect("Unreachable: Entity is valid because it was joined over"));
    ///     }
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn find_with_name<F>(&self, matcher: F, buffer: &mut Vec<Entity>)
    where
        F: FnMut(&str) -> bool;
}

impl<'e, D> FindNamed for Storage<'e, Named, D>
where
    D: Deref<Target = MaskedStorage<Named>>,
{
    fn find_named<S>(&self, s: S) -> Option<Entity>
    where
        S: AsRef<str>,
    {
        let entities = self.fetched_entities();

        self.unprotected_storage()
            .cache
            .map
            .get(s.as_ref())
            .map(|i| entities.entity(*i))
    }

    fn find_with_name<F>(&self, mut matcher: F, buffer: &mut Vec<Entity>)
    where
        F: FnMut(&str) -> bool,
    {
        use specs::Join;

        let iter = (self.fetched_entities(), &*self)
            .join()
            .filter(|(_, n)| matcher(n.name()))
            .map(|(e, _)| e);

        buffer.extend(iter);
    }
}

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
/// # extern crate amethyst;
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
/// # extern crate amethyst;
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
/// # extern crate amethyst;
/// use amethyst::core::{FindNamed, Named};
/// use amethyst::ecs::prelude::*;
///
/// pub struct NameSystem;
///
/// impl<'s> System<'s> for NameSystem {
///     type SystemData = (
///         ReadStorage<'s, Named>,
///     );
///
///     fn run(&mut self, names: Self::SystemData) {
///         if let Some(robot) = names.find_named("Robot") {
///             println!("Entity found: {:?}", robot);
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
    /// # extern crate amethyst;
    /// use amethyst::core::Named;
    ///
    /// let name_component = Named::new("Super Cool Entity");
    /// ```
    ///
    /// From a dynamic string:
    ///
    /// ```
    /// # extern crate amethyst;
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

    /// Borrows the name `str` stored in the `name` field.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Component for Named {
    type Storage = CachedStorage<NameCache, DenseVecStorage<Self>>;
}

/// An easy way to name an `Entity` and give it a `Named` `Component`.
pub trait WithNamed
where
    Self: Sized,
{
    /// Adds a name to the entity being built. This method is available for
    /// the `EntityBuilder` type and will add a `Named` component to it.
    /// See the `Named` type for usage examples.
    ///
    /// # Conflict behavior
    ///
    /// There can be two entities with the same `Named` component, however,
    /// only the last inserted will be returned by `FindNamed::find_named`.
    /// If you want to retrieve all entities with a given name, you need to join
    /// over all entities and filter for the name (or use the `find_with_name`
    /// convenience method).
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

#[derive(Debug, Default)]
pub struct NameCache {
    map: HashMap<Cow<'static, str>, u32>,
}

impl Cache<Named> for NameCache {
    fn on_get(&self, _: u32, _: &Named) {}

    fn on_update(&mut self, id: u32, val: &Named) {
        self.map.insert(val.name.clone(), id);
    }

    fn on_remove(&mut self, _: u32, val: Named) -> Named {
        self.map.remove(&val.name);

        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use named::FindNamed;
    use specs::prelude::*;

    #[test]
    fn test_named_exists() {
        let mut world = World::new();
        world.register::<Named>();

        let a = world.create_entity().named("A").build();

        let b = world.create_entity().named("B").build();

        let storage = world.read_storage::<Named>();
        assert_eq!(storage.find_named("A"), Some(a));
        assert_eq!(storage.find_named("B"), Some(b));
    }

    #[test]
    fn test_named_new_gen() {
        let mut world = World::new();
        world.register::<Named>();

        let a = world.create_entity().named("A").build();

        assert_eq!(a.id(), 0);

        let b = world.create_entity().named("B").build();

        {
            let storage = world.read_storage::<Named>();
            assert_eq!(storage.find_named("A"), Some(a));
            assert_eq!(storage.find_named("B"), Some(b));
        }

        world.delete_entity(a);
        let a = world.create_entity().build();

        // Let's make sure we used the same id.
        assert_eq!(a.id(), 0);

        {
            let storage = world.read_storage::<Named>();
            assert_eq!(storage.find_named("A"), None);
            assert_eq!(storage.find_named("B"), Some(b));
        }

        world.delete_entity(a);
        let a = world.create_entity().named("A").build();

        {
            let storage = world.read_storage::<Named>();
            assert_eq!(storage.find_named("A"), Some(a));
            assert_eq!(storage.find_named("B"), Some(b));
        }
    }

    #[test]
    fn test_named_non_existent() {
        let mut world = World::new();
        world.register::<Named>();

        {
            let storage = world.read_storage::<Named>();
            assert_eq!(storage.find_named("A"), None);
            assert_eq!(storage.find_named("B"), None);
            assert_eq!(storage.find_named("C"), None);
        }

        let a = world.create_entity().named("A").build();

        let b = world.create_entity().named("B").build();

        {
            let storage = world.read_storage::<Named>();
            assert_eq!(storage.find_named("A"), Some(a));
            assert_eq!(storage.find_named("B"), Some(b));
            assert_eq!(storage.find_named("C"), None);
        }

        world.delete_entity(b).unwrap();

        {
            let storage = world.read_storage::<Named>();
            assert_eq!(storage.find_named("A"), Some(a));
            assert_eq!(storage.find_named("B"), None);
            assert_eq!(storage.find_named("C"), None);
        }

        world.entities().delete(a).unwrap();

        {
            let storage = world.read_storage::<Named>();
            // Change is not applied yet
            assert_eq!(storage.find_named("A"), Some(a));
            assert_eq!(storage.find_named("B"), None);
            assert_eq!(storage.find_named("C"), None);
        }

        world.maintain();

        {
            let storage = world.read_storage::<Named>();
            // Change is not applied yet
            assert_eq!(storage.find_named("A"), None);
            assert_eq!(storage.find_named("B"), None);
            assert_eq!(storage.find_named("C"), None);
        }
    }
}
