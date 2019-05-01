use derivative::Derivative;
use rand::{self, distributions::Alphanumeric, Rng};
use std::{
    collections::{
        hash_map::{Keys, Values, ValuesMut},
        HashMap,
    },
    fmt::Display,
    hash::Hash,
    ops::Index,
};

/// A widget is an object that keeps track of all components and entities
/// that make up an element of the user interface. Using the widget_components!
/// macro, it's possible to generate methods that let you easily retrieve
/// all components for a widget, and basically annotate which components the
/// widget will definitely or maybe contain.
/// Widgets are stored in their respective `Widgets` resource and referred to
/// by their associated Id type. A widget will generally only contain fields
/// for the entity Ids it consist of.
pub trait Widget {}

/// A WidgetId is the type by which the different widgets of a type will be
/// differentiated when you create and retrieve them. Generally you'll want something
/// here that can generate a random id with a low chance of collision, since
/// auto generation will be used whenever you don't explicitly don't provide an
/// id to widget builders.
/// It's possible to use something like a bare enum as a WidgetId, but be aware
/// that whenever you're not supplying a WidgetId, you'll probably overwrite an
/// existing widget that had the same default id.
pub trait WidgetId: Clone + PartialEq + Eq + Hash + Send + Sync + Display + 'static {
    /// Generate a new widget id. This function can optionally be passed the last ID
    /// that was generated, to make sequential ids possible.
    fn generate(last: &Option<Self>) -> Self;
}

impl WidgetId for u32 {
    fn generate(last: &Option<Self>) -> Self {
        last.map(|last| last + 1).unwrap_or(0)
    }
}

impl WidgetId for u64 {
    fn generate(last: &Option<Self>) -> Self {
        last.map(|last| last + 1).unwrap_or(0)
    }
}

impl WidgetId for String {
    fn generate(_: &Option<Self>) -> Self {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect()
    }
}

/// Widgets is an alias for a HashMap containing widgets mapped by their
/// respective Id type. It's meant to be used as a resource for every widget type.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Widgets<T: Widget, I: WidgetId = u32> {
    items: HashMap<I, T>,
    last_key: Option<I>,
}

impl<T, I> Widgets<T, I>
where
    T: Widget,
    I: WidgetId,
{
    /// Adds a widget to the map and returns the ID that was created
    /// for it.
    pub fn add(&mut self, widget: T) -> I {
        let id = I::generate(&self.last_key);
        self.items.insert(id.clone(), widget);
        id
    }

    /// Adds a widget with a specified ID. If a widget with the given
    /// ID already existed, the replaced widget will be returned.
    pub fn add_with_id(&mut self, id: I, widget: T) -> Option<T> {
        self.items.insert(id, widget)
    }

    /// Retrieves a widget by its ID.
    pub fn get(&self, id: I) -> Option<&T> {
        self.items.get(&id)
    }

    /// Mutably retrieves a widget by its ID.
    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.items.get_mut(&id)
    }

    /// Provides an iterator over all widgets.
    pub fn widgets(&self) -> Values<'_, I, T> {
        self.items.values()
    }

    /// Provides a mutable iterator over all widgets.
    pub fn widgets_mut(&mut self) -> ValuesMut<'_, I, T> {
        self.items.values_mut()
    }

    /// Provides an iterator over all IDs included in the resource.
    pub fn ids(&self) -> Keys<'_, I, T> {
        self.items.keys()
    }
}

impl<T, I> Index<I> for Widgets<T, I>
where
    T: Widget,
    I: WidgetId,
{
    type Output = T;
    fn index(&self, id: I) -> &Self::Output {
        &self.items[&id]
    }
}

/// Helper macro used inside `widget_components`
#[macro_export]
macro_rules! define_widget_component_fn_impl {
    ( (has $t:ty as $name:ident on $entity:ident) ) => {
        $crate::paste::item! {
            /// Get a reference to the $t component for this widget.
            pub fn [<get_ $name>]<'a>(
                &self,
                storage: &'a amethyst_core::ecs::prelude::ReadStorage<'a, $t>
            ) -> &'a $t {
                // TODO: Better error message
                storage.get(self.$entity)
                    .expect("Component should exist on entity")
            }
        }

        $crate::paste::item! {
            /// Get a mutable reference to the $t component for this widget.
            pub fn [<get_ $name _mut>]<'a>(
                &self,
                storage: &'a mut amethyst_core::ecs::prelude::WriteStorage<'a, $t>
            ) -> &'a mut $t {
                // TODO: Better error message
                storage.get_mut(self.$entity)
                    .expect("Component should exist on entity")
            }
        }
    };

    ( (maybe_has $t:ty as $name:ident on $entity:ident) ) => {
        $crate::paste::item! {
            /// Get a reference to the $t component for this widget if it exists,
            /// `None` otherwise.
            pub fn [<get_ $name _maybe>]<'a>(
                &self,
                storage: &'a amethyst_core::ecs::prelude::ReadStorage<'a, $t>
            ) -> Option<&'a $t> {
                storage.get(self.$entity)
            }
        }

        $crate::paste::item! {
            /// Get a mutable reference to the $t component for this widget
            /// if it exists, `None` otherwise.
            pub fn [<get_ $name _mut_maybe>]<'a>(
                &self,
                storage: &'a mut amethyst_core::ecs::prelude::WriteStorage<'a, $t>
            ) -> Option<&'a mut $t> {
                storage.get_mut(self.$entity)
            }
        }
    };
}

/// This macro allows you to define what components can be attached to a widget,
/// and will automatically generate getters for all components you specify.
/// For each component, you are required to specify which entity handle the
/// component will be attached to.
#[macro_export]
macro_rules! define_widget {
    ($t:ident =>
        entities: [$($field:tt),*]
        components: [$($component:tt),*]
    ) => {
        /// A $t widget, containing references to its associated entities.
        #[derive(Debug, Clone)]
        pub struct $t {
            $(
                /// `$field` Entity
                pub $field: $crate::Entity
            ),*
        }

        impl $crate::Widget for $t {}

        impl $t {
            /// Create a new $t widget from its associated entities.
            pub fn new(
                $($field: $crate::Entity),*
            ) -> Self {
                Self {
                    $($field),*
                }
            }

            $($crate::define_widget_component_fn_impl!{ $component })*
        }
    };
}
