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

Systems in legion / Amethyst are slightly different. Rather than describing the
behavior of a single instance (e.g., a single enemy in your game), they describe
the behavior of all components of a specific type (all enemies). This makes
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

```rust
# use amethyst::core::transform::TransformBundle;
# use amethyst::prelude::*;
# use amethyst::window::DisplayConfig;
# use amethyst::input::InputBundle;
# use amethyst_utils::application_root_dir;
# macro_rules! env {
#   ($x:expr) => {
#       ""
#   };
# }
use amethyst::input::InputBundle;
fn main() -> amethyst::Result<()> {
    // -- snip --

#   let app_root = application_root_dir()?;
#   let path = "./config/display.ron";
#   let config = DisplayConfig::load(&path)?;
#   let assets_dir = "assets";
#   struct Pong;
#   impl SimpleState for Pong {}
#   let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            InputBundle::new().with_bindings_from_file(app_root.join("config/bindings.ron"))?,
        )
    // -- snip --

    let mut game = Application::new(assets_dir, Pong, game_data)?;
#   // game.run();
#   Ok(())
# }
```

For `InputBundle`, the parameter type determines how `axes` and `actions`
are identified in the `bindings.ron` file
(in this example, `String`s are used; e.g. `"left_paddle"`).

At this point, we're ready to write a system that reads input from the
`InputHandler`, and moves the paddles accordingly. First, we'll create a
directory called `systems` under `src` to hold all our systems. We'll use a
module to collect and export each of our systems to the rest of the
application. Here's our `mod.rs` for `src/systems`:

```rust
pub mod paddle;
```

Now we're ready to implement the `PaddleSystem` in `systems/paddle.rs`:

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
    core::transform::Transform,
    ecs::SystemBuilder,
    input::{get_input_axis_simple, InputHandler},
    prelude::*,
};

// You'll have to mark PADDLE_HEIGHT as public in pong.rs
use crate::pong::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT};

pub struct PaddleSystem;

impl System for PaddleSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PaddleSystem")
                .with_query(<(&Paddle, &mut Transform)>::query())
                .read_resource::<InputHandler>()
                .build(move |_commands, world, input, query_paddles| {
                    // -- todo --
                }),
        )
    }
}
# fn main() {}
```

Alright, there's quite a bit going on here!

We create a unit struct `PaddleSystem` and implement the `System` trait for it.
The `System` trait is provided by legion and contains just one method, `build`:

```rust
# use amethyst::prelude::*;
// legion's System trait
pub trait System {
    /// builds the Runnable part of System
    fn build(self) -> Box<dyn ParallelRunnable + 'static>;
}
```

Inside `build`, we describe the data our System needs, and our System's behavior.
Ultimately, `build` returns a `Runnable` struct, which legion uses during runtime 
to run our System.

> ### What's a `Box<dyn ParallelRunnable>`?
> The `Runnable` trait exposes a `run` method, which legion invokes for us during runtime. 
> A `ParallelRunnable` is just a `Runnable` which is also `Send` and `Sync`. Finally, to 
> support any type which implements `ParallelRunnable`, the `build` method returns a `Box<dyn ParallelRunnable>`.

First, we call `SystemBuilder::new` with a name: `"PaddleSystem"`. (Our system's name
is just for debugging and visualization purposes.) 

Second, we invoke `.with_query` with the argument `<(&Paddle, &mut Transform)>::query()`. In
this context, the type `<(&Paddle, &mut Transform)>` declares a **view**. A view describes
the kind of data that our system needs. This view says that our system reads `Paddle`
components and mutates `Transform` components. Legion provides the `query`
method to construct a query from the view that we've declared.

Third, we invoke `.read_resource::<InputHandler>()`. This method declares that our system reads
the `InputHandler` resource, so that we can read the input state.

Finally, we invoke `build` with a closure. This closure accepts four arguments: `|_commands, world, input, query_paddles|`. We're not using `_commands` for now. The `world` contains our 
entities and their component data. `input` is the `InputHandler` resource, which we asked for
above with `.read_resource::<InputHandler>()`. Finally, `query_paddles` contains the query
that we described above with `.with_query`. Our closure has all of the data we requested,
and it's time to write the system's behavior!

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
# use amethyst::{
#     core::transform::Transform,
#     ecs::SystemBuilder,
#     input::{get_input_axis_simple, InputHandler},
#     prelude::*,
# };
# 
# // You'll have to mark PADDLE_HEIGHT as public in pong.rs
# use crate::pong::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT};
# 
# pub struct PaddleSystem;
# 
# impl System for PaddleSystem {
#    fn build(self) -> Box<dyn ParallelRunnable> {
#        Box::new(
            SystemBuilder::new("PaddleSystem")
                .with_query(<(&Paddle, &mut Transform)>::query())
                .read_resource::<InputHandler>()
                .build(move |_commands, world, input, query_paddles| {
                    for (paddle, transform) in query_paddles.iter_mut(world) {
                        // read input
                        // move paddle
                    }
                }),
#        )
#    }
# }
# fn main() {}
```

Now that we have access to the storages of the components we want, we can iterate
over them. We invoke the `iter_mut` method on our query to perform a join operation 
over the `Transform` and `Paddle` storages. This will iterate over all entities that 
have both a `Paddle` and `Transform` attached to them, and give us access to the actual components, immutable for the `Paddle` and mutable for the `Transform`.

> There are many other ways to use storages. For example, you can use them to get
> a reference to the component of a specific type held by an entity, or simply
> iterate over them without joining. However, in practice, your most common use will
> be to join over multiple storages as it is rare to have a system affect
> only one specific component.

## Reading the input

Our system needs to know the current state of the input, so we can decide how to move the paddles.
Let's use the `InputHandler` we asked for!

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
# use amethyst::{
#     core::transform::Transform,
#     ecs::SystemBuilder,
#     input::{get_input_axis_simple, InputHandler},
#     prelude::*,
# };
# 
# // You'll have to mark PADDLE_HEIGHT as public in pong.rs
# use crate::pong::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT};
# 
# pub struct PaddleSystem;
# 
# impl System for PaddleSystem {
#    fn build(self) -> Box<dyn ParallelRunnable> {
#        Box::new(
#           SystemBuilder::new("PaddleSystem")
#               .with_query(<(&Paddle, &mut Transform)>::query())
#               .read_resource::<InputHandler>()
#               .build(move |_commands, world, input, query_paddles| {
                    for (paddle, transform) in query_paddles.iter_mut(world) {
                        let movement = match paddle.side {
                            Side::Left => get_input_axis_simple(&Some("left_paddle".into()), input),
                            Side::Right => {
                                get_input_axis_simple(&Some("right_paddle".into()), input)
                            }
                        };
                    }
#               }),
#        )
#    }
# }
# fn main() {}
```

Amethyst provides the `get_input_axis_simple` function. We take the axis name we wrote earlier
in `bindings.ron`, convert it to `Some` clone-on-write smart pointer, and pass it into `get_input_axis_simple` to get the state of our axis.

## Modifying the transform

Let's make our system update the position of the paddle. To do this, we'll modify the y
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
# use amethyst::{
#     core::transform::Transform,
#     ecs::SystemBuilder,
#     input::{get_input_axis_simple, InputHandler},
#     prelude::*,
# };
# 
# // You'll have to mark PADDLE_HEIGHT as public in pong.rs
# use crate::pong::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT};
# 
# pub struct PaddleSystem;
# 
# impl System for PaddleSystem {
#    fn build(self) -> Box<dyn ParallelRunnable> {
#        Box::new(
#           SystemBuilder::new("PaddleSystem")
#               .with_query(<(&Paddle, &mut Transform)>::query())
#               .read_resource::<InputHandler>()
#               .build(move |_commands, world, input, query_paddles| {
                    for (paddle, transform) in query_paddles.iter_mut(world) {
                        let movement = match paddle.side {
                            Side::Left => get_input_axis_simple(&Some("left_paddle".into()), input),
                            Side::Right => {
                                get_input_axis_simple(&Some("right_paddle".into()), input)
                            }
                        };
                    }
                    let scaled_amount = 1.2 * movement;
                    let paddle_y = transform.translation().y;
                    transform.set_translation_y((paddle_y + scaled_amount));
#               }),
#        )
#    }
# }
# fn main() {}
```

This is our first attempt at moving the paddles: we take the movement and
scale it by some factor to make the motion seem smooth. In a real game, we
would use the time elapsed between frames to determine how far to move the
paddle, so that the behavior of the game would not be tied to the game's
framerate. Amethyst provides [`amethyst::core::timing::Time`][doc_time]
for that purpose. For now, our current approach will suffice.

Let's add our system to the game! In `main.rs`:

```rust
mod pong;
mod systems; // Add our systems mod

use amethyst::{
    // --snip--
#   assets::LoaderBundle,
#   core::transform::TransformBundle,
#   input::InputBundle,
#   prelude::*,
#   renderer::{
#       plugins::{RenderFlat2D, RenderToWindow},
#       rendy::hal::command::ClearColor,
#       types::DefaultBackend,
#       RenderingBundle,
#   },
#   utils::application_root_dir,
};
# use systems::paddle::PaddleSystem;

# use crate::pong::Pong;

fn main() -> amethyst::Result<()> {
#   amethyst::start_logger(Default::default());
#
#   let app_root = application_root_dir()?;
#   let display_config_path = app_root.join("config/display.ron");
#   let assets_dir = app_root.join("assets/");
#
#   let mut dispatcher = DispatcherBuilder::default();
    // -- snip--
    dispatcher
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            InputBundle::new().with_bindings_from_file(app_root.join("config/bindings.ron"))?,
        )
        // We have now added our own system, the PaddleSystem, defined in systems/paddle.rs
        .add_system(PaddleSystem)
        // -- snip--
#       .add_bundle(
#           RenderingBundle::<DefaultBackend>::new()
#               // The RenderToWindow plugin provides all the scaffolding for opening a window and
#               // drawing on it
#               .with_plugin(
#                   RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
#                       float32: [0.0, 0.0, 0.0, 1.0],
#                   }),
#               )
#               // RenderFlat2D plugin is used to render entities with `SpriteRender` component.
#               .with_plugin(RenderFlat2D::default()),
#       );

#   let game = Application::new(assets_dir, Pong, dispatcher)?;
#   game.run();
#   Ok(())
}
```

If you run the game now, you'll notice the paddles can "fall" off the edges of the game area.

To fix this, we need to limit the paddle's movement to the arena border with
a minimum and maximum value. Since the anchor point of the paddle is in
its center, we also need to offset that limit by half the height
of the paddle to prevent the paddle going halfway out of the screen.
Therefore, we will clamp the **y** value of the transform from
`ARENA_HEIGHT - PADDLE_HEIGHT * 0.5` (the top of the arena minus the offset)
to `PADDLE_HEIGHT * 0.5` (the bottom of the arena plus the offset).

Our `build` function in `systems/paddle.rs` should now look something like this:

```rust
# use amethyst::{
#     core::transform::Transform,
#     ecs::SystemBuilder,
#     input::{get_input_axis_simple, InputHandler},
#     prelude::*,
# };
# const PADDLE_HEIGHT: f32 = 16.0;
# const PADDLE_WIDTH: f32 = 4.0;
# const ARENA_HEIGHT: f32 = 100.0;
# const ARENA_WIDTH: f32 = 100.0;
# enum Side {
#   Left,
#   Right,
# }
# pub struct Paddle {
#   side: Side,
# }
#
# pub struct PaddleSystem;
impl System for PaddleSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PaddleSystem")
                .with_query(<(&Paddle, &mut Transform)>::query())
                .read_resource::<InputHandler>()
                .build(move |_commands, world, input, query_paddles| {
                    for (paddle, transform) in query_paddles.iter_mut(world) {
                        let movement = match paddle.side {
                            Side::Left => get_input_axis_simple(&Some("left_paddle".into()), input),
                            Side::Right => {
                                get_input_axis_simple(&Some("right_paddle".into()), input)
                            }
                        };
                        let scaled_amount = 1.2 * movement;
                        let paddle_y = transform.translation().y;
                        transform.set_translation_y(
                            (paddle_y + scaled_amount)
                                .min(ARENA_HEIGHT - PADDLE_HEIGHT * 0.5)
                                .max(PADDLE_HEIGHT * 0.5),
                        );
                    }
                }),
        )
    }
}
```

When you run the game, the paddles should now stay in the arena. Nice work!

## Summary

In this chapter, we added an input handler to our game, so that we
could capture keypresses. We then created a system that would interpret these
keypresses, and move our game's paddles accordingly. In the next chapter, we'll
explore another key concept in real-time games: time. We'll make our game aware
of time, and add a ball for our paddles to bounce back and forth.

[doc_bindings]: https://docs.amethyst.rs/master/amethyst_input/struct.Bindings.html
[doc_time]: https://docs.amethyst.rs/master/amethyst_core/timing/struct.Time.html
