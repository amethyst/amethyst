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

Systems in Specs / Amethyst are slightly different. Rather than describing the
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
# use amethyst_utils::application_root_dir;
# macro_rules! env {
#   ($x:expr) => {
#       ""
#   };
# }
# fn main() -> amethyst::Result<()> {
    use amethyst::input::InputBundle;

#   let app_root = application_root_dir()?;
    let binding_path = app_root.join("config").join("bindings.ron");

    let input_bundle = InputBundle::new().with_bindings_from_file(binding_path)?;

#   let path = "./config/display.ron";
#   let config = DisplayConfig::load(&path)?;
#   let assets_dir = "assets";
#   struct Pong;
#   impl SimpleState for Pong {}
    let game_data = DispatcherBuilder::default()
        .add_bundle(TransformBundle::new())
        .add_bundle(input_bundle);
    // ..

    let mut game = Application::new(assets_dir, Pong, game_data)?;
    //game.run();
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
pub use self::paddle::PaddleSystem;

mod paddle;
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
use amethyst::core::{Transform};
use amethyst::ecs::{System, World};
use amethyst::input::InputHandler;

// You'll have to mark PADDLE_HEIGHT as public in pong.rs
use crate::pong::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT};

pub struct PaddleSystem;

impl System for PaddleSystem {
    type SystemData = (
        .write_component::<Transform>()
       .read_component::<Paddle>(),
        .read_resource::<InputHandler>(),
    );

    fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
        for (paddle, transform) in (&paddles, &mut transforms).join() {
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
                    println!("Side {:?} moving {}", side_name, mv_amount);
                }
            }
        }
    }
}
# fn main() {}
```

Alright, there's quite a bit going on here!

We create a unit struct `PaddleSystem`, and with the `SystemDesc` derive. This
is short for **System Descriptor**. In Amethyst, systems may need to access
resources from the `World` in order to be instantiated. For each `System`, an
implementation of the `SystemDesc` trait must be provided to specify the logic
to instantiate the `System`. For `System`s that do not require special
instantiation logic, the `SystemDesc` derive automatically implements the
`SystemDesc` trait on the system type itself.

Next, we implement the `System` trait for it with the lifetime of the components
on which it operates. Inside the implementation, we define the data the system
operates on in the `SystemData` tuple: `WriteStorage`, `ReadStorage`, and
`Read`. More specifically, the generic types we've used here tell us that the
`PaddleSystem` mutates `Transform` components, `.write_component::<Transform>`, it
reads `Paddle` components,.read\_component::<Paddle>()`, and also accesses the `InputHandler`resource we created earlier, using the`Read\`
structure.

> For `InputHandler`, make sure the parameter type is the same
> as the one used to create the `InputBundle` earlier.

Now that we have access to the storages of the components we want, we can iterate
over them. We perform a join operation over the `Transform` and `Paddle`
storages. This will iterate over all entities that have both a `Paddle`
and `Transform` attached to them, and give us access to the actual components,
immutable for the `Paddle` and mutable for the `Transform`.

> There are many other ways to use storages. For example, you can use them to get
> a reference to the component of a specific type held by an entity, or simply
> iterate over them without joining. However, in practice, your most common use will
> be to join over multiple storages as it is rare to have a system affect
> only one specific component.

> Please also note that it is possible to join over storages using multiple threads
> by using `par_join` instead of `join`, but here the overhead introduced is not
> worth the gain offered by parallelism.

Let's add this system to our `DispatcherBuilder` in `main.rs`:

```rust
mod systems; // Import the module
```

```rust
# use amethyst::core::transform::TransformBundle;
# use amethyst::input::StringBindings;
# use amethyst::prelude::*;
# use amethyst::window::DisplayConfig;
fn main() -> amethyst::Result<()> {
    // --snip--

#   let path = "./config/display.ron";
#   let config = DisplayConfig::load(&path)?;
#   mod systems {
# 
#       use amethyst::core::ecs::{System, World};
# 
#       pub struct PaddleSystem;
#       impl System for PaddleSystem {
#           fn build(mut self) -> Box<dyn ParallelRunnable> {}
#       }
#   }
#   let input_bundle = amethyst::input::InputBundle::new();
    let game_data = DispatcherBuilder::default()
// ...
.add_bundle(TransformBundle::new())?
.add_bundle(input_bundle)?
.with(systems::PaddleSystem, "paddle_system", &["input_system"]) // Add this line
// ...
#;
#   let assets_dir = "/";
#   struct Pong;
#   impl SimpleState for Pong {}
#   let mut game = Application::new(assets_dir, Pong, game_data)?;
#   Ok(())
}
```

Take a look at the `with` method call. Here, we're not adding a bundle, we're adding
a system alone. We provide an instance of the system, a string representing its name
and a list of dependencies. The dependencies are the names of the systems that
must be run before our newly added system. Here, we require the `input_system` to be run as we will use the user's input to move the paddles, so we need to have this
data be prepared. The `input_system` key itself is defined in the standard `InputBundle`.

## Modifying the transform

If we run the game now, we'll see the console print our keypresses.
Let's make it update the position of the paddle. To do this, we'll modify the y
component of the transform's translation.

```rust
# use amethyst::core::Transform;
# use amethyst::ecs::{System, World};
# use amethyst::input::InputHandler;
# enum Side {
#   Left,
#   Right,
# }
# pub struct Paddle {
#   side: Side,
# }
#
# pub struct PaddleSystem;
#
# impl System for PaddleSystem {
#   type SystemData = (
#       .write_component::<Transform>()
#     .read_component::<Paddle>(),
#       .read_resource::<InputHandler>(),
#   );
    fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
        for (paddle, transform) in (&paddles, &mut transforms).join() {
            let movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };
            if let Some(mv_amount) = movement {
                let scaled_amount = 1.2 * mv_amount as f32;
                transform.prepend_translation_y(scaled_amount);
            }
        }
    }
# }
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
# use amethyst::core::Transform;
# use amethyst::ecs::{System, World};
# use amethyst::input::InputHandler;
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
# impl System for PaddleSystem {
#   type SystemData = (
#       .write_component::<Transform>()
#     .read_component::<Paddle>(),
#       .read_resource::<InputHandler>(),
#   );
    fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
        for (paddle, transform) in (&paddles, &mut transforms).join() {
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
    }
# }
```

## Automatic set up of resources by a system.

You might remember that we had troubles because Amethyst requires us
to `register` storage for `Paddle` before we could use it.

Now that we have a system in place that uses the `Paddle` component,
we no longer need to manually register it with the `world`: the system
will take care of that for us, as well as set up the storage.

```rust
# use amethyst::assets::Handle;
# use amethyst::ecs::World;
# use amethyst::prelude::*;
# use amethyst::renderer::SpriteSheet;
# struct Paddle;
# fn initialize_paddles(world: &mut World, spritesheet: Handle<SpriteSheet>) {}
# fn initialize_camera(world: &mut World) {}
# fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
#   unimplemented!()
# }
# struct MyState;
# impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        // Load the spritesheet necessary to render the graphics.
        let sprite_sheet_handle = load_sprite_sheet(world);

        world.register::<Paddle>(); // <<-- No longer needed

        initialize_paddles(world, sprite_sheet_handle);
        initialize_camera(world);
    }
# }
```

## Summary

In this chapter, we added an input handler to our game, so that we
could capture keypresses. We then created a system that would interpret these
keypresses, and move our game's paddles accordingly. In the next chapter, we'll
explore another key concept in real-time games: time. We'll make our game aware
of time, and add a ball for our paddles to bounce back and forth.

[doc_bindings]: https://docs.amethyst.rs/master/amethyst_input/struct.Bindings.html
[doc_time]: https://docs.amethyst.rs/master/amethyst_core/timing/struct.Time.html
