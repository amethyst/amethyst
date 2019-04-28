# How to Define State Dispatcher: Internal

This guide explains how to define a state-specific `Dispatcher` whose `System`s are determined by the `State` and is unaffected by input from a previous `State` or the application. For simplicity sake we'll be using the `SimpleState` trait for our custom state.

We'll start off by creating our `State` with a `dispatcher` field.

```rust,edition2018,no_run,noplaypen
extern crate amethyst;

use amethyst::{
    ecs::prelude::*, 
    prelude::*
};

pub struct CustomDispatcherState<'a, 'b> {
    /// State specific dispatcher.
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for CustomDispatcherState<'a, 'b> {}

impl<'a, 'b> CustomDispatcherState<'a, 'b> {
    fn new() -> Self {
        CustomDispatcherState {
            dispatcher: None,
        }
    }
}
```

The `CustomDispatcherState` requires two annotations (`'a` and `'b`) to satisfy the lifetimes of `Dispatcher`.

Then we'll initialise the `Dispatcher` with the required `System`s inside the `State`s `update(..)` method. This way all resources required by the `System`s will be available.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# 
# use amethyst::{
#     ecs::prelude::*, 
#     prelude::*
# };
# 
# pub struct CustomDispatcherState<'a, 'b> {
#     /// State specific dispatcher.
#     dispatcher: Option<Dispatcher<'a, 'b>>,
# }
# 
impl<'a, 'b> SimpleState for CustomDispatcherState<'a, 'b> {
    fn on_start(&mut self, data: StateData<GameData>) {
        let mut dispatcher_builder = DispatcherBuilder::new();

        /// Register `System`s manually.
        dispatcher_builder.add(MovementSystem, "movement_systen", &["input_system"]);
        dispatcher_builder.add(CollisionSystem, "collision_system", &[]);

        // If you would like to register bundles,
        // you may do so here.
        SomeSystemsBundle::default()
            .build(&mut dispatcher_builder)
            .expect("Failed to register SomeSystemsBundle");

        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut data.world.res);
        self.dispatcher = Some(dispatcher);
    }
}
# 
# impl<'a, 'b> CustomDispatcherState<'a, 'b> {
#     fn new() -> Self {
#         CustomDispatcherState {
#             dispatcher: None,
#         }
#     }
# }
```

To finish it off we have to execute the `Dispatcher` during the `State`s `update(..)` method. This ensures that the `System`s are actually executed.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# 
# use amethyst::{ecs::prelude::*, prelude::*};
# 
# pub struct CustomDispatcherState<'a, 'b> {
#     /// State specific dispatcher.
#     dispatcher: Option<Dispatcher<'a, 'b>>,
# }
# 
impl<'a, 'b> SimpleState for CustomDispatcherState<'a, 'b> {
    fn on_start(&mut self, data: StateData<GameData>) {
        let mut dispatcher_builder = DispatcherBuilder::new();

        /// Register `System`s manually.
        dispatcher_builder.add(MovementSystem, "movement_systen", &["input_system"]);
        dispatcher_builder.add(CollisionSystem, "collision_system", &[]);

        // If you would like to register bundles,
        // you may do so here.
        SomeSystemsBundle::default()
            .build(&mut dispatcher_builder)
            .expect("Failed to register SomeSystemsBundle");

        let mut dispatcher = dispatcher_builder.build();
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
#     fn new() -> Self {
#         CustomDispatcherState {
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