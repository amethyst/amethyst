# Moving the paddles

In the previous chapter, we learned about the relationship between entities and
components, and how they represent the "things" in our games. This chapter
introduces Systems - the S in "ECS". Systems are objects that represent
operations over entities, or more specifically, combinations of components.
Let's add a system that moves the paddles based on user input.

A system is nothing more than a function that runs once each frame and
potentially makes some changes to components. If you've used other game
engines, this probably sounds familiar: Unity engine calls these objects
`MonoBehaviour`s and Unreal engine calls them `Actor`s, but these all represent
the same basic idea.

Systems in Amethyst (Legion) are slightly different. Rather than describing the
behavior of a single instance (e.g., a single enemy in your game), they describe
the behavior of all components of a specific type (e.g. all enemies). This makes
your code more modular, easier to test, and makes it run faster.

Let's get started.

## Capturing user input

To capture user input, we'll need to introduce a few more files to our game.
Let's start by creating a config file under the `config` directory of our
project, called `bindings.ron`, which will contain a RON representation
of the [amethyst\_input::Bindings][doc_bindings] struct:

```ron
(
  axes: {
    "left_paddle": Emulated(pos: Key(W), neg: Key(S)),
    "right_paddle": Emulated(pos: Key(Up), neg: Key(Down)),
  },
  actions: {},
)
```

In Amethyst, inputs can either be axes (a range that represents an analog
controller stick or relates two buttons as opposite ends of a range), or actions
(also known as scalar input - a button that is either pressed or not).
In this file, we're creating the inputs to move each paddle up (`pos:`) or down
(`neg:`) on the vertical axis: **W** and **S** for the left paddle, and the **Up**
and **Down** arrow keys for the right paddle.
We name them `"left_paddle"` and `"right_paddle"`, which will allow us to
refer to them by name in the code when we will need to read their respective values
to update positions.

Next, we'll add an `InputBundle` to the game's `Application` object, that
contains an `InputHandler` system which captures inputs, and maps them to the
axes we defined. Let's make the following changes to `main.rs`.

```rust ,should_panic
use amethyst::input::InputBundle;
# use amethyst::{prelude::*, window::DisplayConfig};
# use amethyst_utils::application_root_dir;
# 
# struct Pong;
# impl SimpleState for Pong {}
fn main() -> amethyst::Result<()> {
#   let app_root = application_root_dir()?;
#   let mut game_data = DispatcherBuilder::default();
    // ...
    let binding_path = app_root.join("config").join("bindings.ron");
    game_data.add_bundle(InputBundle::new().with_bindings_from_file(binding_path)?);
    // ...
#   Ok(())
}
```

For `InputBundle`, the parameter type determines how `axes` and `actions`
are identified in the `bindings.ron` file
(in this example, `String`s are used; e.g. `"left_paddle"`).

At this point, we're ready to write a system that reads input from the
`InputHandler`, and moves the paddles accordingly. First, we'll create a
directory called `systems` under `src` to hold all our systems. We'll use a
module to collect and export each of our systems to the rest of the
application. Here's our `mod.rs` for `src/systems`:

```rust ,ignore
mod paddle;

pub use self::paddle::PaddleSystem;
```

We're finally ready to implement the `PaddleSystem` in `systems/paddle.rs`:

```rust
# mod pong {
#   pub enum Side {
#       Left,
#       Right,
#   }
#   pub struct Paddle {
#       pub side: Side,
#   }
# 
#   pub const ARENA_HEIGHT: f32 = 100.0;
#   pub const PADDLE_HEIGHT: f32 = 16.0;
# }
# 
use amethyst::{
    core::Transform,
    ecs::{IntoQuery, ParallelRunnable, System, SystemBuilder, World},
    input::InputHandler,
};
use pong::{Paddle, Side};

pub struct PaddleSystem;

impl System for PaddleSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PaddleSystem")
                .write_component::<Transform>()
                .read_component::<Paddle>()
                .read_resource::<InputHandler>()
                .with_query(<(&Paddle, &mut Transform)>::query())
                .build(|_, world, input, query| {
                    for (paddle, transform) in query.iter_mut(world) {
                        let movement = match paddle.side {
                            Side::Left => input.axis_value("left_paddle"),
                            Side::Right => input.axis_value("right_paddle"),
                        };
                        if let Some(mv_amount) = movement {
                            if mv_amount != 0.0 {
                                let side_name = match paddle.side {
                                    Side::Left => "left",
                                    Side::Right => "right",
                                };
                                println!("{:?} side moving {}", side_name, mv_amount);
                            }
                        }
                    }
                }),
        )
    }
}
```

Alright, there's quite a bit going on here!

First, we implement the `System` trait for `PaddleSystem`.
Inside the implementation, we specify which components the system operates on with `.read_component`, `.write_component` and `.with_query`.
We also specify resources needed with `.with_resource`.

Now that we have access to the storages of the components we want, we can iterate
over them. We perform a query operation over the `Transform` and `Paddle`
storages with `query.iter_mut(world)`. This will iterate over all entities that have both a `Paddle`
and `Transform` attached to them, and give us access to the actual components,
immutable for the `Paddle` and mutable for the `Transform`.

> There are many other ways to use components. For example, you can get
> a reference to the component of a specific type held by an entity, or
> iterate over them without a query. However, in practice, your most common use will
> be to join over multiple storages as it is rare to have a system affect
> only one specific component.

Let's add this system to our `DispatcherBuilder` in `main.rs`:

```rust ,ignore
mod systems;

// ...

fn main() -> amethyst::Result<()> {
    // ...
    game_data.add_bundle(InputBundle::new().with_bindings_from_file(binding_path)?);
    // ...
    game_data.add_system(systems::PaddleSystem);
    // ...
}
```

Take a look at the `add_system` method call. We provide an instance of the system.
Systems run in the order they are added to `game_data` each frame.
Be sure to add the `PaddleSystem` after the `InputBundle` so we can respond to input on the same frame it is received.

## Modifying the transform

If we run the game now, we'll see the console print our keypresses.
Let's make it update the position of the paddle. To do this, we'll modify the y
component of the transform's translation.

```rust
# mod pong {
#   pub enum Side {
#       Left,
#       Right,
#   }
#   pub struct Paddle {
#       pub side: Side,
#   }
# 
#   pub const ARENA_HEIGHT: f32 = 100.0;
#   pub const PADDLE_HEIGHT: f32 = 16.0;
# }
# 
use amethyst::{
    core::Transform,
    ecs::{IntoQuery, ParallelRunnable, System, SystemBuilder, World},
    input::InputHandler,
};
use pong::{Paddle, Side};

pub struct PaddleSystem;

impl System for PaddleSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PaddleSystem")
                .write_component::<Transform>()
                .read_component::<Paddle>()
                .read_resource::<InputHandler>()
                .with_query(<(&Paddle, &mut Transform)>::query())
                .build(|_, world, input, query| {
                    for (paddle, transform) in query.iter_mut(world) {
                        let movement = match paddle.side {
                            Side::Left => input.axis_value("left_paddle"),
                            Side::Right => input.axis_value("right_paddle"),
                        };
                        if let Some(mv_amount) = movement {
                            let scaled_amount = 1.2 * mv_amount as f32;
                            transform.prepend_translation_y(scaled_amount);
                        }
                    }
                }),
        )
    }
}
```

This is our first attempt at moving the paddles: we take the movement and
scale it by some factor to make the motion seem smooth. In a real game, we
would use the time elapsed between frames to determine how far to move the
paddle, so that the behavior of the game would not be tied to the game's
framerate. Amethyst provides you with [`amethyst::core::timing::Time`][doc_time]
for that purpose, but for now current approach should suffice.
If you run the game now, you'll notice the paddles are able to "fall" off the edges of the game area.

To fix this, we need to limit the paddle's movement to the arena border with
a minimum and maximum value. But as the anchor point of the paddle is in
the middle of the sprite, we also need to offset that limit by half the height
of the sprite for the paddles not to go halfway out of the screen.
Therefore, we will clamp the **y** value of the transform from
`ARENA_HEIGHT - PADDLE_HEIGHT * 0.5` (the top of the arena minus the offset)
to `PADDLE_HEIGHT * 0.5` (the bottom of the arena plus the offset).

Our run function should now look something like this:

```rust
# mod pong {
#   pub enum Side {
#       Left,
#       Right,
#   }
#   pub struct Paddle {
#       pub side: Side,
#   }
# 
#   pub const ARENA_HEIGHT: f32 = 100.0;
#   pub const PADDLE_HEIGHT: f32 = 16.0;
# }
# 
use amethyst::{
    core::Transform,
    ecs::{IntoQuery, ParallelRunnable, System, SystemBuilder, World},
    input::InputHandler,
};
use pong::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT};

pub struct PaddleSystem;

impl System for PaddleSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PaddleSystem")
                .write_component::<Transform>()
                .read_component::<Paddle>()
                .read_resource::<InputHandler>()
                .with_query(<(&Paddle, &mut Transform)>::query())
                .build(|_, world, input, query| {
                    for (paddle, transform) in query.iter_mut(world) {
                        let movement = match paddle.side {
                            Side::Left => input.axis_value("left_paddle"),
                            Side::Right => input.axis_value("right_paddle"),
                        };
                        if let Some(mv_amount) = movement {
                            let scaled_amount = 1.2 * mv_amount as f32;
                            let paddle_y = transform.translation().y;
                            transform.set_translation_y(
                                (paddle_y + scaled_amount)
                                    .min(ARENA_HEIGHT - PADDLE_HEIGHT * 0.5)
                                    .max(PADDLE_HEIGHT * 0.5),
                            );
                        }
                    }
                }),
        )
    }
}
```

## Summary

In this chapter, we added an input handler to our game, so that we
could capture keypresses. We then created a system that would interpret these
keypresses, and move our game's paddles accordingly. In the next chapter, we'll
explore another key concept in real-time games: time. We'll make our game aware
of time, and add a ball for our paddles to bounce back and forth.

[doc_bindings]: https://docs.amethyst.rs/master/amethyst_input/struct.Bindings.html
[doc_time]: https://docs.amethyst.rs/master/amethyst_core/timing/struct.Time.html
