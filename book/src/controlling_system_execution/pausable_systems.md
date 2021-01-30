# Pausable Systems

Custom `GameData` and state-specific `System`s are great when it comes to handling groups of `System`. But when it comes single `System`s or a group of `System`s spread over multiple `Dispatcher`s or `State`s, pausable `Sytem`s come in handy.

Pausable `System`s can be enabled or disabled depending on the value of a`Resource` registered to your `World`. When this value changes, the state of your `System` changes as well.

Let's get started by creating a new `Resource` that represents the state of our game.

```rust
#[derive(PartialEq)]
pub enum CurrentState {
    Running,
    Paused,
}

impl Default for CurrentState {
    fn default() -> Self {
        CurrentState::Paused
    }
}
```

We'll use this `enum` `Resource` to control whether or not our `System` is running. Next we'll register our `System` and set it as pausable.

```rust
#
# use amethyst::{
#     ecs::*,
#     prelude::*,
# };
# 
# #[derive(PartialEq)]
# pub enum CurrentState {
#     Running,
#     Paused,
# }
# 
# impl Default for CurrentState {
#     fn default() -> Self {
#         CurrentState::Paused
#     }
# }
#
# #[derive(Default)] struct MovementSystem;
# 
# impl System for MovementSystem {
#
#   fn run(&mut self, data: Self::SystemData) {}
# }
# let mut dispatcher = DispatcherBuilder::new();
dispatcher.add(
    MovementSystem::default().pausable(CurrentState::Running),
    "movement_system",
    &["input_system"],
);
```

`pausable(CurrentState::Running)` creates a wrapper around our `System` that controls its execution depending on the `CurrentState` `Resource` registered with the `World`. As long as the value of the `Resource` is set to `CurrentState::Running`, the `System` is executed.

To register the `Resource` or change its value, we can use the following code:

```rust
# use amethyst::prelude::*;
# #[derive(PartialEq)]
# pub enum CurrentState {
#   Running,
#   Paused,
# }
# 
# impl Default for CurrentState {
#   fn default() -> Self {
#       CurrentState::Paused
#   }
# }
# 
struct GameplayState;

impl SimpleState for GameplayState {
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
#       let my_condition = true;
        if (my_condition) {
            *data.world.write_resource::<CurrentState>() = CurrentState::Paused;
        }

        Trans::None
    }
}
```

However, this cannot be done inside the pausable `System` itself. A pausable `System` can only access its pause `Resource` with immutable `Read` and cannot modify the value, thus the `System` cannot decide on its own if it should run on not. This has to be done from a different location.
