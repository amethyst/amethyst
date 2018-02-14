# Moving the paddles

In the previous chapter, we learned about the relationship between entities and components, and how they represent the "things" in our games. This chapter introduces Systems— the S in "ECS". Systems are objects that represent operations over entities, or more specifically, combinations of components. Let's add a system that moves the paddles based on user input.

A system is nothing more than a function that runs once each frame and potentially makes some changes to components. If you've used other game engines, this probably sounds familiar: Unity engine calls these objects as `MonoBehaviour`s and Unreal engine calls them `Actor`s, but these all represent the same basic idea.

Systems in specs/amethyst are slightly different. Rather than describe the functionality of a single component, they describe the functionality of all components of a specific type. This difference impacts the runtime performance of a game, since it lets us optimize the way the components are stored with cache-locality in mind. It also means we can make systems run in parallel when they don't share any data between them.

Let's get started.

## Capturing user input
To capture user input, we'll need to introduce a few more files to our game. Let's start by creating a resource file under the `resources` directory of our project, called `bindings.ron`:

```ron,ignore
(
  axes: {
    "left_paddle": (pos: Key(W), neg: Key(S)),
    "right_paddle": (pos: Key(Up), neg: Key(Down))
  },
  actions: {},
)
```

In amethyst, inputs can be either scalar inputs (eg, a button that is pressed or not), or axes (eg, a range that relates two buttons as opposite ends of a range). In this file, we're creating two axes: W,S will move the left paddle up and down, and the Up and Down arrow keys will move the right paddle up and down.

Next, we'll add an input bundle to the game's `Application` object, that contains an input handler system which captures inputs and maps them to the axes we defined. Let's make the following changes to `main.rs`.

```rust,ignore
use amethyst::input::InputBundle;

let input_bundle = InputBundle::<String, String>::new()
.with_bindings_from_file("./resources/bindings.ron");

let mut game = Application::build("./", Pong)?
.with_bundle(TransformBundle::new())?
.with_bundle(RenderBundle::new())?
.with_bundle(input_bundle)?
.with_local(RenderSystem::build(pipe, Some(config))?)
.build()?;
```

At this point, we're ready to write a system that reads input from the input handler, and moves the paddles accordingly. First, we'll create a directory called `systems` under `src` to hold all our systems. We'll use a module to collect and export each of our systems to the rest of the application. Here's our `mod.rs` for `src/systems`:

```rust,ignore
mod paddle;

pub use self::paddle::PaddleSystem;
```

We're finally ready to implement the PaddleSystem:

```rust,ignore
use pong::{Paddle, Side};
use amethyst::ecs::{System, Join, Fetch};
use amethyst::input::InputHandler;
use amethyst::core::LocalTransform;
use amethyst::ecs::{ReadStorage, WriteStorage};

pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
  type SystemData = (
     WriteStorage<'s, LocalTransform>,
     ReadStorage<'s, Paddle>,
     Fetch<'s, InputHandler<String, String>>,
   );

   fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
      for paddle in (&paddles).join() {
        let movement = match paddle.side {
          Side::Left => input.axis_value("left_paddle"),
          Side::Right => input.axis_value("right_paddle"),
        };
        if let Some(mv_amount) = movement {
          if mv_amount != 0.0 {
            println!("Side {:?} moving {}", paddle.side, mv_amount);
          }
        }
      }
    }
  }
```

Note: We had to make our Paddle and Side public in `pong.rs`

Let's review what our system does, because there's quite a bit there.

We create a unit struct, called PaddleSystem, and implement the System trait for it. The trait specifies the lifetime of the components on which it operates. Inside the implementation, we define the `SystemData` the system operates on, a tuple of `WriteStorage`, `ReadStorage`, and `Fetch`. More specifically, the generic types we've used here tell us that the `PaddleSystem` mutates LocalTransform components, `WriteStorage<'s, LocalTransform>`, it reads `Paddle` components, `ReadStorage<'s, Paddle>`, and also accesses the `InputHandler<String, String>` resource we created earlier, using the `Fetch` trait.

It's worth noting an important difference between the objects our system accesses. A system will iterate over entities that contain one of each of the components that it specifies in its SystemData. Going back to our example, the PaddleSystem will ignore any entity that is missing a `Paddle`, `LocalTransform` or both.

## Modifying the transform

If we run the game now, we'll see the console print our keypresses. Let's make it update the position of the paddle. To do this, we'll modify the y component of the transform's translation.

```rust,ignore
  fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
    for (paddle, mut transform) in (&paddles, &mut transforms).join() {
      let movement = match paddle.side {
        Side::Left => input.axis_value("left_paddle"),
        Side::Right => input.axis_value("right_paddle"),
      };
      if let Some(mv_amount) = movement {
        let scaled_amount = (1.0 / 60.0) * mv_amount as f32;
        transform.translation[1] += scaled_amount;
      }
    }
  }
```

This is our first attempt at moving the paddles: we take the movement, and scale it by some factor to make the motion seem smooth. If you run the game now, you'll notice the paddles are able to fall off the edges of the game area.

To fix this, we'll clamp the translation's y component to be at least -1.0 and at most `1.0 - PADDLE_HEIGHT` (since the translation indicates the paddle's bottom edge).

Our run function should now look something like this:

```rust,ignore
  fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
    for (paddle, mut transform) in (&paddles, &mut transforms).join() {
      let movement = match paddle.side {
        Side::Left => input.axis_value("left_paddle"),
        Side::Right => input.axis_value("right_paddle"),
      };
      if let Some(mv_amount) = movement {
        let scaled_amount = (1.0 / 60.0) * mv_amount as f32;
        transform.translation[1] = (transform.translation[1] + scaled_amount)
        .min(1.0 - PADDLE_HEIGHT)
        .max(-1.0);
      }
    }
  }
```

## Summary
In this chapter, we created added an input handler to our game, so that we could capture keypresses. We then added a system that would interpret these keypresses, and move our game's paddles accordingly. In the next chapter, we'll explore another key concept in real-time games: time. We'll make our game aware of time, and add a ball for our paddles to bounce back and forth.
