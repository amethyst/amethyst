# How to Define State Dispatcher

This guide explains how to define a state-specific `Dispatcher` whose `System`s are only executed within the context of a defined `State`.

First of all we required a `DispatcherBuilder`. The `DispatcherBuilder` handles the actual creation of the `Dispatcher` and the assignment of `System`s to our `Dispatcher`.

```rust
#
# use amethyst::{
#     ecs::*,
#     prelude::*,
# };
# 
let mut dispatcher_builder = DispatcherBuilder::new();
```

To add `System`s to the `DispatcherBuilder` we use a similar syntax to the one we used to add `System`s to `GameData`.

```rust
#
# use amethyst::{
#     ecs::*,
#     prelude::*,
# };
#
# struct MoveBallsSystem; struct MovePaddlesSystem;
# impl System for MoveBallsSystem { fn run(&mut self, _: ()) {} }
# impl System for MovePaddlesSystem { fn run(&mut self, _: ()) {} }
let mut dispatcher_builder = DispatcherBuilder::new();

dispatcher_builder.add(MoveBallsSystem, "move_balls_system", &[]);
dispatcher_builder.add(MovePaddlesSystem, "move_paddles_system", &[]);
```

Alternatively we can add `Bundle`s of `System`s to our `DispatcherBuilder` directly.

```rust
#
# use amethyst::{
#     core::bundle::SystemBundle,
#     ecs::{DispatcherBuilder, World},
#     prelude::*,
# };
# #[derive(Default)] struct PongSystemsBundle;
# impl SystemBundle<'a, 'b> for PongSystemsBundle {
#     fn build(self, _: &mut World, _: &mut DispatcherBuilder) -> Result<(), amethyst::Error> {
#         Ok(())
#     }
# }
#
# let mut world = World::default();
let mut dispatcher_builder = DispatcherBuilder::new();

PongSystemsBundle::default()
    .build(&mut world, &mut dispatcher_builder)
    .expect("Failed to register PongSystemsBundle");
```

The `DispatcherBuilder` can be initialized and populated wherever desired, be it inside the `State` or in an external location. However, the `Dispatcher` needs to modify the `World`s resources in order to initialize the resources used by its `System`s. Therefore, we need to defer building the `Dispatcher` until we can access the `World`. This is commonly done in the `State`s `on_start` method. To showcase how this is done, we'll create a `SimpleState` with a `dispatcher` field and a `on_start` method that builds the `Dispatcher`.

```rust
# use amethyst::{core::ArcThreadPool, ecs::*, prelude::*};
# 
# struct MoveBallsSystem;
# struct MovePaddlesSystem;
# impl System for MoveBallsSystem {
#   fn run(&mut self, _: ()) {}
# }
# impl System for MovePaddlesSystem {
#   fn run(&mut self, _: ()) {}
# }
# 
#[derive(Default)]
pub struct CustomState<'a, 'b> {
    /// The `State` specific `Dispatcher`, containing `System`s only relevant for this `State`.
    dispatcher: Option<Dispatcher>,
}

impl<'a, 'b> SimpleState for CustomState<'a, 'b> {
    fn on_start(&mut self, mut data: StateData<'_, GameData>) {
        let world = &mut data.world;

        // Create the `DispatcherBuilder` and register some `System`s that should only run for this `State`.
        let mut dispatcher_builder = DispatcherBuilder::new();
        dispatcher_builder.add(MoveBallsSystem, "move_balls_system", &[]);
        dispatcher_builder.add(MovePaddlesSystem, "move_paddles_system", &[]);

        // Build and setup the `Dispatcher`.
        let mut dispatcher = dispatcher_builder
            .with_pool((*resources.get::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
    }
}
```

By default, the dispatcher will create its own pool of worker threads to execute systems in, but Amethyst's main dispatcher already has a thread pool setup and configured. As reusing it is more efficient, we pull the global pool from the world and attach the dispatcher to it with `.with_pool()`.

The `CustomState` requires two annotations (`'a` and `'b`) to satisfy the lifetimes of the `Dispatcher`. Now that we have our `Dispatcher` we need to ensure that it is executed. We do this in the `State`s `update` method.

```rust
# use amethyst::{ecs::*, prelude::*};
# 
# #[derive(Default)]
# pub struct CustomState<'a, 'b> {
#   /// The `State` specific `Dispatcher`, containing `System`s only relevant for this `State`.
#   dispatcher: Option<Dispatcher>,
# }
# struct MoveBallsSystem;
# struct MovePaddlesSystem;
# impl System for MoveBallsSystem {
#   fn run(&mut self, _: ()) {}
# }
# impl System for MovePaddlesSystem {
#   fn run(&mut self, _: ()) {}
# }
# 
impl<'a, 'b> SimpleState for CustomState<'a, 'b> {
    fn on_start(&mut self, mut data: StateData<'_, GameData>) {
        let world = &mut data.world;

        // Create the `DispatcherBuilder` and register some `System`s that should only run for this `State`.
        let mut dispatcher_builder = DispatcherBuilder::new();
        dispatcher_builder.add(MoveBallsSystem, "move_balls_system", &[]);
        dispatcher_builder.add(MovePaddlesSystem, "move_paddles_system", &[]);

        // Build and setup the `Dispatcher`.
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        Trans::None
    }
}
```

Now, any `System`s in this `State`-specific `Dispatcher` will only run while this `State` is active and the `update` method is called.
