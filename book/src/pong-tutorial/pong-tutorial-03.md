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

Systems in specs/Amethyst are slightly different. Rather than describe the
behavior of a single instance (eg, a single enemy in your game), they describe
the behavior of all components of a specific type (all enemies). This makes
your code more modular, easier to test, and makes it run faster.

Let's get started.

## Capturing user input

To capture user input, we'll need to introduce a few more files to our game.
Let's start by creating a resource file under the `resources` directory of our
project, called `bindings_config.ron`:

```ron,ignore
(
  axes: {
    "left_paddle": Emulated(pos: Key(W), neg: Key(S)),
    "right_paddle": Emulated(pos: Key(Up), neg: Key(Down)),
  },
  actions: {},
)
```

In Amethyst, inputs can be either scalar inputs (a button that is either
pressed or not), or axes (a range that represents an analog controller stick or
relates two buttons as opposite ends of a range).
In this file, we're creating two axes: W and S will move the
left paddle up and down, and the Up and Down arrow keys will move the right
paddle up and down.

Next, we'll add an input bundle to the game's `Application` object, that
contains an input handler system which captures inputs and maps them to the
axes we defined. Let's make the following changes to `main.rs`.

```rust,ignore
use amethyst::input::InputBundle;

let binding_path = format!(
    "{}/resources/bindings_config.ron",
    env!("CARGO_MANIFEST_DIR")
);

let input_bundle = InputBundle::<String, String>::new().with_bindings_from_file(binding_path)?;

let game_data = GameDataBuilder::default()
    .with_bundle(TransformBundle::new())?
    .with_bundle(RenderBundle::new(pipe, Some(config)))?
    .with_bundle(input_bundle)?;
let mut game = Application::new("./", Pong, game_data)?;
game.run();
```

At this point, we're ready to write a system that reads input from the
`InputHandler`, and moves the paddles accordingly. First, we'll create a
directory called `systems` under `src` to hold all our systems. We'll use a
module to collect and export each of our systems to the rest of the
application. Here's our `mod.rs` for `src/systems`:

```rust,ignore
mod paddle;

pub use self::paddle::PaddleSystem;
```

We're finally ready to implement the `PaddleSystem` in `systems/paddle.rs`:

```rust,ignore
use amethyst::core::transform::components::Transform;
use amethyst::ecs::{Join, Read, ReadStorage, System, WriteStorage};
use amethyst::input::InputHandler;
use pong::{Paddle, Side};

pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
  type SystemData = (
    WriteStorage<'s, Transform>,
    ReadStorage<'s, Paddle>,
    Read<'s, InputHandler<String, String>>,
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
```

Note: We had to make our Paddle and Side public in `pong.rs`

Now lets add this system to our `GameDataBuilder` in `main.rs`:

```rust,ignore
mod systems;

// in the run() function
let game_data = GameDataBuilder::default()
    .with_bundle(TransformBundle::new())?
    .with_bundle(RenderBundle::new(pipe, Some(config)))?
    .with_bundle(input_bundle)?
    .with(systems::PaddleSystem, "paddle_system", &["input_system"]); // Add this line
```

Take a look at the `with` method call. Here, we're not adding a bundle, we're adding
a system alone. We provide an instance of the system, a string representing its name
and a list of dependencies. The dependencies are the names of the systems that
must be ran before our newly added system. Here, we require the `input_system` to be
ran as we will use the user's input to move the paddles, so we need to have this
data be prepared.

Back in `paddle.rs`, let's review what our system does, because there's quite a bit there.

We create a unit struct, called `PaddleSystem`, and implement the `System`
trait for it. The trait specifies the lifetime of the components on which it
operates. Inside the implementation, we define the `SystemData` the system
operates on, a tuple of `WriteStorage`, `ReadStorage`, and `Read`. More
specifically, the generic types we've used here tell us that the `PaddleSystem`
mutates `LocalTransform` components, `WriteStorage<'s, LocalTransform>`, it
reads `Paddle` components, `ReadStorage<'s, Paddle>`, and also accesses the
`InputHandler<String, String>` resource we created earlier, using the `Read`
structure.

Then, now that we have access to the storages of the components we want, we can
iterate over them. We perform a join operation between the `Transform` and `Paddle`
storages. This will iterate over all entities that have both a `Paddle` and `Transform`
attached to them, and give us access to the actual components, immutably for the
`Paddle` and mutably for the `Transform`.

> There are many other ways to use storages. For example, you can use them to get
> a reference to the component of a specific type held by an entity, or simply
> iterate over them without joining. However in practice, your most common use will
> be to join over multiple storages as it is rare to have a system affect
> only one specific component.

> Please also note that it is possible to join over storages using multiple threads
> by using `par_join` instead of `join`, but here the overhead introduced is not
> worth the gain offered by parallelism.

## Modifying the transform

If we run the game now, we'll see the console print our keypresses. Let's
make it update the position of the paddle. To do this, we'll modify the y
component of the transform's translation.

```rust,ignore
  fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
    for (paddle, mut transform) in (&paddles, &mut transforms).join() {
      let movement = match paddle.side {
        Side::Left => input.axis_value("left_paddle"),
        Side::Right => input.axis_value("right_paddle"),
      };
      if let Some(mv_amount) = movement {
        let scaled_amount = 1.2 * mv_amount as f32;
        transform.translation[1] += scaled_amount;
      }
    }
  }
```

This is our first attempt at moving the paddles: we take the movement, and
scale it by some factor to make the motion seem smooth. In a real game, we
would use the time elapsed between frames to determine how far to move the
paddle, so that the behavior of the game would not be tied to the game's
framerate, but this will do for now. If you run the game now, you'll notice
the paddles are able to "fall" off the edges of the game area.

To fix this, we'll make sure the paddle's anchor point never gets out of the
arena. But as the anchor point is in the middle of the sprite, we also need
to add a margin for the paddle to not go halfway out of the screen.
Therefore, we will border the y value of the transform from
`ARENA_HEIGHT - PADDLE_HEIGHT * 0.5` (the top of the screen but a little bit
lower) to `PADDLE_HEIGHT * 0.5` (the bottom of the screen but a little bit higher).


Our run function should now look something like this:

```rust,ignore
  fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
    for (paddle, mut transform) in (&paddles, &mut transforms).join() {
      let movement = match paddle.side {
        Side::Left => input.axis_value("left_paddle"),
        Side::Right => input.axis_value("right_paddle"),
      };
      if let Some(mv_amount) = movement {
        let scaled_amount = 1.2 * mv_amount as f32;
        transform.translation[1] = (transform.translation[1] + scaled_amount)
          .min(ARENA_HEIGHT - PADDLE_HEIGHT * 0.5)
          .max(PADDLE_HEIGHT * 0.5);
      }
    }
  }
```

Note: For the above to work, we'll have to mark `PADDLE_HEIGHT` and `ARENA_HEIGHT`
as being public in `pong.rs`, and then import it in `paddle.rs`.

## Summary
In this chapter, we added an input handler to our game, so that we
could capture keypresses. We then created a system that would interpret these
keypresses, and move our game's paddles accordingly. In the next chapter, we'll
explore another key concept in real-time games: time. We'll make our game aware
of time, and add a ball for our paddles to bounce back and forth.
