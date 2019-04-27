# How to Define State Dispatcher: External

This guide explains how to define a state-specific dispatcher whose `System`s are passed in externally, which means at the moment of state struct creation. This is used when the list of `System`s is determined by user choices at runtime. For simplicity sake we'll be using the `SimpleState` trait for our custom state. 

For convenience we'll create a builder for our `State`. This way we'll be able to create our `State` and register `System`s as follows:

```rust,edition2018,no_run,noplaypen
CustomDispatcherBuilder::new()
    .with(MovementSystem, "movement_systen", &["input_system"])
    .with(CollisionSystem, "collision_system", &[])
    .build(); 
```

We'll start off by creating our `State`. 

```rust,edition2018,no_run,noplaypen
extern crate amethyst;

use amethyst::{
    ecs::prelude::*,
    prelude::*,
};

pub struct CustomDispatcherState<'a, 'b> {
    /// The `State` specific `DispatcherBuilder`. This will be used build the actual `Dispatcher`.
    dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,

    /// The `State` specific `Dispatcher`, containing `System`s only relevant for this `State`.
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for CustomDispatcherState<'a, 'b> {}

impl<'a, 'b> CustomDispatcherState<'a, 'b> {
    fn new(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
        Self {
            dispatcher_builder: Some(dispatcher_builder),
            dispatcher: None,
        }
    }
}
```

The `CustomDispatcherState` requires two lifetime annotations (`'a` and `'b`) for use in the `DispatcherBuilder` and `Dispatcher`.

Then we'll create the builder for `CustomDispatcherState` that initialises the `DispatcherBuilder`, populates it with the desired `System`s and builds the state.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     ecs::prelude::*,
#     prelude::*,
# };
# 
/// Builder for the `CustomDispatcherState`.
///
/// This allows you to specify which `System`s you want to run within the state.
pub struct CustomDispatcherStateBuilder<'a, 'b> {
    /// The `State` specific `DispatcherBuilder`.
    dispatcher_builder: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> CustomDispatcherStateBuilder<'a, 'b> {

    /// Returns a new `CustomDispatcherStateBuilder`.
    pub fn new() -> Self {
        CustomDispatcherStateBuilder {
            dispatcher_builder: DispatcherBuilder::new(),
        }
    }

    /// Registers a `System` with the dispatcher builder.
    ///
    /// # Parameters
    ///
    /// * `system`: `System` to register.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with<Sys>(mut self, system: Sys, name: &str, deps: &[&str]) -> Self
        where Sys: for<'c> System<'c> + Send + 'a,
    {
        self.dispatcher_builder.add(system, name, deps);
        self
    }

    /// Builds and returns the `CustomDispatcherState`.
    pub fn build(self) -> CustomDispatcherState<'a, 'b> {
        CustomDispatcherState::new(self.dispatcher_builder)
    }
}
# 
# pub struct CustomDispatcherState<'a, 'b> {
#     /// The `State` specific `DispatcherBuilder`. This will be used build the actual `Dispatcher`.
#     dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
# 
#     /// The `State` specific `Dispatcher`, containing `System`s only relevant for this `State`.
#     dispatcher: Option<Dispatcher<'a, 'b>>,
# }
# 
# impl<'a, 'b> SimpleState for CustomDispatcherState<'a, 'b> {}
# 
# impl<'a, 'b> CustomDispatcherState<'a, 'b> {
#     fn new(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
#         Self {
#             dispatcher_builder: Some(dispatcher_builder),
#             dispatcher: None,
#         }
#     }
# }
```

This enables us to create the `State` and register `System`s as mentioned above.

Now we have to actually build the `Dispatcher` within the `State`.  In order to initialize the resources used by `System`s, building the `Dispatcher` needs to modify the `World`'s resources. Therefore, we need to defer building the actual `Dispatcher` until we can access them. Let's update our `on_start` method:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# 
# use amethyst::{
#     ecs::prelude::*,
#     prelude::*,
# };
# 
# /// Builder for the `CustomDispatcherState`.
# ///
# /// This allows you to specify which `System`s you want to run within the state.
# pub struct CustomDispatcherStateBuilder<'a, 'b> {
#     /// The `State` specific `DispatcherBuilder`.
#     dispatcher_builder: DispatcherBuilder<'a, 'b>,
# }
# 
# impl<'a, 'b> CustomDispatcherStateBuilder<'a, 'b> {
#     /// Returns a new `CustomDispatcherStateBuilder`.
#     pub fn new() -> Self {
#         CustomDispatcherStateBuilder {
#             dispatcher_builder: DispatcherBuilder::new(),
#         }
#     }
# 
#     /// Registers a `System` with the dispatcher builder.
#     ///
#     /// # Parameters
#     ///
#     /// * `system`: `System` to register.
#     /// * `name`: Name to register the system with, used for dependency ordering.
#     /// * `deps`: Names of systems that must run before this system.
#     pub fn with<Sys>(mut self, system: Sys, name: &str, deps: &[&str]) -> Self
#         where Sys: for<'c> System<'c> + Send + 'a,
#     {
#         self.dispatcher_builder.add(system, name, deps);
#         self
#     }
# 
#     /// Builds and returns the `CustomDispatcherState`.
#     pub fn build(self) -> CustomDispatcherState<'a, 'b> {
#         CustomDispatcherState::new(self.dispatcher_builder)
#     }
# }
# 
# pub struct CustomDispatcherState<'a, 'b> {
#     /// The `State` specific `DispatcherBuilder`. This will be used to build the actual dispatcher.
#     dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
# 
#     /// The `State` specific `Dispatcher`, containing `System`s only relevant for this `State`.
#     dispatcher: Option<Dispatcher<'a, 'b>>,
# }
# 
impl<'a, 'b> SimpleState for CustomDispatcherState<'a, 'b> {
    fn on_start(&mut self, data: StateData<GameData>) {
        let mut dispatcher = self.dispatcher_builder
            .take()
            .expect("Expected `dispatcher_builder` to exist when `dispatcher` is not yet built.")
            .build(); // We build the dispatcher itself
        dispatcher.setup(&mut data.world.res); // We register the resources it uses into the world
        self.dispatcher = Some(dispatcher);
    }
}
# 
# impl<'a, 'b> CustomDispatcherState<'a, 'b> {
#     fn new(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
#         Self {
#             dispatcher_builder: Some(dispatcher_builder),
#             dispatcher: None,
#         }
#     }
# }
```

To finish it off we have to execute the `Dispatcher` during the `State`s `update(..)` method. This ensures that the `System`s are actually executed.
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# 
# use amethyst::{
#     ecs::prelude::*,
#     prelude::*,
# };
# 
# /// Builder for the `CustomDispatcherState`.
# ///
# /// This allows you to specify which `System`s you want to run within the state.
# pub struct CustomDispatcherStateBuilder<'a, 'b> {
#     /// The `State` specific `DispatcherBuilder`.
#     dispatcher_builder: DispatcherBuilder<'a, 'b>,
# }
# 
# impl<'a, 'b> CustomDispatcherStateBuilder<'a, 'b> {
#     /// Returns a new `CustomDispatcherStateBuilder`.
#     pub fn new() -> Self {
#         CustomDispatcherStateBuilder {
#             dispatcher_builder: DispatcherBuilder::new(),
#         }
#     }
# 
#     /// Registers a `System` with the dispatcher builder.
#     ///
#     /// # Parameters
#     ///
#     /// * `system`: `System` to register.
#     /// * `name`: Name to register the system with, used for dependency ordering.
#     /// * `deps`: Names of systems that must run before this system.
#     pub fn with<Sys>(mut self, system: Sys, name: &str, deps: &[&str]) -> Self
#         where Sys: for<'c> System<'c> + Send + 'a,
#     {
#         self.dispatcher_builder.add(system, name, deps);
#         self
#     }
# 
#     /// Builds and returns the `CustomDispatcherState`.
#     pub fn build(self) -> CustomDispatcherState<'a, 'b> {
#         CustomDispatcherState::new(self.dispatcher_builder)
#     }
# }
# 
# pub struct CustomDispatcherState<'a, 'b> {
#     /// The `State` specific `DispatcherBuilder`. This will be used to build the actual dispatcher.
#     dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
# 
#     /// The `State` specific `Dispatcher`, containing `System`s only relevant for this `State`.
#     dispatcher: Option<Dispatcher<'a, 'b>>,
# }
# 
impl<'a, 'b> SimpleState for CustomDispatcherState<'a, 'b> {
    fn on_start(&mut self, data: StateData<GameData>) {
        let mut dispatcher = self.dispatcher_builder
            .take()
            .expect("Expected `dispatcher_builder` to exist when `dispatcher` is not yet built.")
            .build();
        dispatcher.setup(&mut data.world.res);
        self.dispatcher = Some(dispatcher);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world.res);
        }

        Trans::None
    }
}
# 
# impl<'a, 'b> CustomDispatcherState<'a, 'b> {
#     fn new(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
#         Self {
#             dispatcher_builder: Some(dispatcher_builder),
#             dispatcher: None,
#         }
#     }
# }
```

Now, any systems in the state-specific `Dispatcher` will only run in this `State`. The *application* `Dispatcher` is automatically executed for `SimpleState` implementations. For more complex states that implement other `State` specific traits, the *application* `Dispatcher` has to be explicitly executed on top of the `State` specific `Dispatcher`. This is done by slightly modifying your `State`s `update(..)` method:

```rust,edition2018,no_run,noplaypen
data.data.update(&data.world);
self.dispatcher.as_mut().unwrap().dispatch(&data.world.res);
        
Trans::None
```
