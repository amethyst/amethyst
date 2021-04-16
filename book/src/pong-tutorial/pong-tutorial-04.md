# Making a ball move and bounce

In the previous chapter, we learned how to capture user input
to make things move on the screen by creating a `System` ourselves.
This chapter will reuse all the knowledge we acquired through the
previous chapters to add a new object to our game: a ball that moves
and bounces around!

First, let's define some other useful constants for this chapter in `pong.rs`:

```rust
pub const BALL_VELOCITY_X: f32 = 75.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;
```

This could also be done by using an external config file. This is
especially useful when you want to edit values a lot. Here, we're
keeping it simple.

## Create our next Component: The ball Component!

In `pong.rs`, let's create the `Ball` Component.

```rust
pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}
```

A ball has a velocity and a radius, so we store that information in the component.

Then let's add an `initialize_ball` function the same way we wrote the
`initialize_paddles` function.

```rust
# use amethyst::{
#     assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
#     core::{timing::Time, transform::Transform},
#     prelude::*,
#     renderer::{Camera, SpriteRender, SpriteSheet, Texture},
# };
# pub const ARENA_HEIGHT: f32 = 100.0;
# pub const ARENA_WIDTH: f32 = 100.0;
# pub const PADDLE_HEIGHT: f32 = 16.0;
# pub const PADDLE_WIDTH: f32 = 4.0;
# pub const BALL_VELOCITY_X: f32 = 75.0;
# pub const BALL_VELOCITY_Y: f32 = 50.0;
# pub const BALL_RADIUS: f32 = 2.0;
# pub struct Ball {
#     pub velocity: [f32; 2],
#     pub radius: f32,
# }
/// initializes one ball in the middle-ish of the arena.
fn initialize_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    // Create the translation.
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(
        (ARENA_WIDTH - BALL_RADIUS) * 0.5,
        (ARENA_HEIGHT - BALL_RADIUS) * 0.5,
        0.0,
    );

    // Assign the sprite for the ball
    let sprite_render = SpriteRender::new(sprite_sheet_handle, 1); // ball is the second sprite on the sprite_sheet

    world.push((
        sprite_render,
        Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        },
        local_transform,
    ));
}
```

In [a previous chapter][pong_02_drawing] we saw how to load a sprite sheet
and get things drawn on the screen. Remember sprite sheet information
is stored in `pong_spritesheet.ron`, and the ball sprite was the
second one, whose index is `1`.

Finally, let's make sure the code is working as intended by updating the `on_start` method:

```rust
# use amethyst::{
#     assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
#     core::{timing::Time, transform::Transform},
#     prelude::*,
#     renderer::{Camera, SpriteRender, SpriteSheet, Texture},
# };
# struct Paddle;
# struct Ball;
# fn initialize_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {}
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

        initialize_ball(world, sprite_sheet_handle.clone()); // <- add this line
        initialize_paddles(world, sprite_sheet_handle);
        initialize_camera(world);
    }
# }
```

Don't forget to call `clone` on `sprite_sheet_handle` because `initialize_paddles` and
`initialize_ball` *consume* the handle.

By running the game now, you should be able to see the two paddles and the ball
in the center. In the next section, we're going to make this ball actually move!

## Create systems to make the ball move

We're now ready to implement the `MoveBallsSystem` in `systems/move_balls.rs`:

```rust
# mod pong {
#   pub struct Ball {
#       pub velocity: [f32; 2],
#       pub radius: f32,
#   }
# }
# 
use amethyst::{
    core::{timing::Time, transform::Transform},
    ecs::SystemBuilder,
    prelude::*,
};

use crate::pong::Ball;

pub struct MoveBallsSystem;

impl System for BallSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MoveBallsSystem")
                .with_query(<(&Ball, &mut Transform)>::query())
                .read_resource::<Time>()
                .read_component::<Ball>()
                .write_component::<Transform>()
                .build(move |_commands, world, time, query_balls| {
                    for (ball, local) in query_balls.iter_mut(world) {
                        local.prepend_translation_x(ball.velocity[0] * time.delta_seconds());
                        local.prepend_translation_y(ball.velocity[1] * time.delta_seconds());
                    }
                }),
        )
    }
}
# fn main() {}
```

This system is responsible for moving all balls according to their speed and
the elapsed time. Notice how the `iter_mut()` method is used to iterate over all
ball entities. Here we only have one ball, but if we ever need multiple, the
system will handle them out of the box.
In this system, we also want *framerate independence*.
That is, no matter the framerate, all objects move with the same speed.
To achieve that, a **delta time**, which is the duration since the last frame, is used.
This is commonly known as ["delta timing"][delta_timing].
As you can see in the snippet, to gain access to time passed since the last frame,
you need to use [`amethyst::core::timing::Time`][doc_time], a commonly used
resource. It has a method called `delta_seconds` that does exactly what we want.

Now that our ball can move, let's implement a new System:
`BounceSystem` in `systems/bounce.rs`.
It will be responsible for detecting collisions between balls and
paddles, as well as balls and the top and bottom edges of the arena.
If a collision is detected, the ball bounces off. This is done
by negating the velocity of the `Ball` component on the `x` or `y` axis.

```rust
# mod pong {
#   pub struct Ball {
#       pub velocity: [f32; 2],
#       pub radius: f32,
#   }
# 
#   #[derive(PartialEq, Eq)]
#   pub enum Side {
#       Left,
#       Right,
#   }
# 
#   pub struct Paddle {
#       pub side: Side,
#       pub width: f32,
#       pub height: f32,
#   }
# 
#   pub const ARENA_HEIGHT: f32 = 100.0;
# }
# 
use amethyst::{core::transform::Transform, ecs::SystemBuilder, prelude::*};

use crate::pong::{Ball, Paddle, Side, ARENA_HEIGHT};

pub struct BounceSystem;

impl System for BounceSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("BounceSystem")
                .with_query(<(&mut Ball, &Transform)>::query())
                .with_query(<&Paddle>::query())
                .read_component::<Paddle>()
                .read_component::<Transform>()
                .write_component::<Ball>()
                .build(
                    move |_commands, world, _resources, (query_balls, query_paddles)| {
                        let (mut ball_world, remaining) = world.split_for_query(query_balls);

                        // Check whether a ball collided, and bounce off accordingly.
                        //
                        // We also check for the velocity of the ball every time, to prevent multiple collisions
                        // from occurring.
                        for (ball, transform) in query_balls.iter_mut(&mut ball_world) {
                            let ball_x = transform.translation().x;
                            let ball_y = transform.translation().y;

                            // Bounce at the top or the bottom of the arena.
                            if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                                || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
                            {
                                ball.velocity[1] = -ball.velocity[1];
                            }

                            // Bounce at the paddles.
                            for paddle in query_paddles.iter(&remaining) {
                                // To determine whether the ball has collided with a paddle, we create a larger
                                // rectangle around the current one, by subtracting the ball radius from the
                                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                                // is then within the paddle if its centre is within the larger wrapper
                                // rectangle.
                                if point_in_rect(
                                    ball_x,
                                    ball_y,
                                    paddle.x - paddle.width / 2. - ball.radius,
                                    paddle.y - paddle.height / 2. - ball.radius,
                                    paddle.x + paddle.width / 2. + ball.radius,
                                    paddle.y + paddle.height / 2. + ball.radius,
                                ) && ((paddle.side == Side::Left && ball.velocity[0] < 0.0)
                                    || (paddle.side == Side::Right && ball.velocity[0] > 0.0))
                                {
                                    println!("Bounce!");
                                    ball.velocity[0] = -ball.velocity[0];
                                }
                            }
                        }
                    },
                ),
        )
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
# fn main() {}
```

The following image illustrates how collisions with paddles are checked.

![Collision explanotary drawing](../images/pong_tutorial/pong_paddle_collision.png)

Also, don't forget to add `mod move_balls` and `mod bounce` in `systems/mod.rs`
as well as adding our new systems to the game data:

```rust
# use amethyst::core::transform::TransformBundle;
# use amethyst::input::StringBindings;
# use amethyst::prelude::*;
# use amethyst::window::DisplayConfig;
# fn main() -> amethyst::Result<()> {
#   let path = "./config/display.ron";
#   let config = DisplayConfig::load(&path)?;
#   mod systems {
#       use amethyst::core::ecs::{System, World};
#       pub struct PaddleSystem;
#       impl System for PaddleSystem {
#           fn build(mut self) -> Box<dyn ParallelRunnable> {}
#       }
#       pub struct MoveBallsSystem;
#       impl System for MoveBallsSystem {
#           fn build(mut self) -> Box<dyn ParallelRunnable> {}
#       }
#       pub struct BounceSystem;
#       impl System for BounceSystem {
#           fn build(mut self) -> Box<dyn ParallelRunnable> {}
#       }
#   }
#   let input_bundle = amethyst::input::InputBundle::new();
    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
#       .add_bundle(LoaderBundle)
#       // Add the transform bundle which handles tracking entity positions
#       .add_bundle(TransformBundle)
#       .add_bundle(
#           InputBundle::new().with_bindings_from_file(app_root.join("config/bindings.ron"))?,
#       )
#       .add_system(PaddleSystem)
        // ...other systems...
        .add_system(BallSystem)
        .add_system(BounceSystem);
#   let assets_dir = "/";
#   struct Pong;
#   impl SimpleState for Pong {}
#   let game = Application::new(assets_dir, Pong::default(), dispatcher)?;
#   Ok(())
# }
```

You should now have a ball moving and bouncing off paddles and off the top
and bottom of the screen. However, you will quickly notice that if the ball
goes out of the screen on the right or the left, it never comes back
and the game is over. You might not even see that at all, as the ball might be already
outside of the screen when the window comes up. You might have to dramatically reduce
`BALL_VELOCITY_X` in order to see that in action. This obviously isn't a good solution for an actual game.
To fix that problem and better see what's happening we have to spawn the ball with a slight delay.

## Spawning ball with a delay

The ball now spawns and moves off screen instantly when the game starts. This might be disorienting,
as you might be thrown into the game and lose your first point before you had the time to notice.
We also have to give some time for the operating system and the renderer to initialize the window
before the game starts. Usually, you would have a separate state with a game menu, so this isn't an issue.
Our pong game throws you right into the action, so we have to fix that problem.

Let's delay the first time the ball spawns. This is also a good opportunity to use our game state
struct to actually hold some data.

First, let's add a new method to our state: `update`.
Let's add that `update` method just below `on_start`:

```rust
# use amethyst::prelude::*;
# struct Pong;
# impl SimpleState for Pong {
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        Trans::None
    }
# }
```

That method allows you to transition out of state using its return value.
Here, we do not want to change any state, so we return `Trans::None`.

Now we have to move paddle creation to that method and add some delay to it. Our `update` runs every frame,
so in order to do something only once after a given time, we have to use our local state.
Additionally, notice that `initialize_paddles` requires us to provide the `sprite_sheet_handle`, but it was created
as a local variable inside `on_start`. For that reason, we have to make it a part of the state too.

Let's add some fields to our `Pong` struct:

```rust
# use amethyst::assets::Handle;
#[derive(Default)]
pub struct Pong {
    ball_spawn_timer: Option<f32>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}
```

Our timer is represented by `Option<f32>`, which will count down to zero when available, and be replaced with `None` after
the time has passed. Our sprite sheet handle is also inside `Option` because we can't create it inside `Pong` constructor.
It will be created inside the `on_start` method instead.

We've also added `#[derive(Default)]`, which will automatically implement `Default` trait for us, which allows to create
default empty state. Now let's use that inside our `Application` creation code in `main.rs`:

```rust
# use amethyst::{ecs::World, prelude::*};
# 
# #[derive(Default)]
# struct Pong;
# impl SimpleState for Pong {}
# fn main() -> amethyst::Result<()> {
#   let mut dispatcher = DispatcherBuilder::default();
#   let assets_dir = "/";
#   let world = World::default();
    let mut game = Application::new(assets_dir, Pong::default(), dispatcher)?;
#   Ok(())
# }
```

Now let's finish our timer and ball spawning code. We have to do two things:

- First, we have to initialize our state and remove `initialize_ball` from `on_start`,
- then we have to `initialize_ball` once after the time has passed inside `update`:

```rust
use amethyst::{
    assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
    core::{timing::Time, transform::Transform},
    prelude::*,
    renderer::{Camera, SpriteRender, SpriteSheet, Texture},
};

# struct Paddle;
# struct Ball;
# fn initialize_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {}
# fn initialize_paddles(world: &mut World, spritesheet: Handle<SpriteSheet>) {}
# fn initialize_camera(world: &mut World) {}
# fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
#   unimplemented!()
# }
# #[derive(Default)]
# pub struct Pong {
#   ball_spawn_timer: Option<f32>,
#   sprite_sheet_handle: Option<Handle<SpriteSheet>>,
# }
# 
// -- snip --
impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        // Wait one second before spawning the ball.
        self.ball_spawn_timer.replace(1.0);

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        self.sprite_sheet_handle
            .replace(load_sprite_sheet(data.resources));
        initialize_paddles(world, self.sprite_sheet_handle.clone().unwrap());
        initialize_camera(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        if let Some(mut timer) = self.ball_spawn_timer.take() {
            // If the timer isn't expired yet, subtract the time that passed since the last update.
            {
                let time = data.resources.get::<Time>().unwrap();
                timer -= time.delta_seconds();
            }
            if timer <= 0.0 {
                // When timer expire, spawn the ball
                initialize_ball(data.world, self.sprite_sheet_handle.clone().unwrap());
            } else {
                // If timer is not expired yet, put it back onto the state.
                self.ball_spawn_timer.replace(timer);
            }
        }
        Trans::None
    }
}
```

Now our ball will only show up after a set delay, giving us some breathing room after startup.
This will give us a better opportunity to see what happens to the ball immediately when it spawns.

## Summary

In this chapter, we finally added a ball to our game. As always, the full code
is available under the `pong_tutorial_04` example in the Amethyst repository.
In the next chapter, we'll add a system checking when a player loses the game,
and add a scoring system!

[delta_timing]: https://en.wikipedia.org/wiki/Delta_timing
[doc_time]: https://docs.amethyst.rs/master/amethyst_core/timing/struct.Time.html
[pong_02_drawing]: pong-tutorial-02.html#drawing
