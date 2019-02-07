# Making a ball move and bounce

In the previous chapter, we learned how to capture user input
to make things move on the screen by creating a `System` ourselves.
This chapter will reuse all the knowledge we acquired through the
previous chapters to add a new object to our game: a ball that moves
and bounces around!

First, let's define some other useful constants for this chapter in `pong.rs`:

```rust,edition2018,no_run,noplaypen
pub const BALL_VELOCITY_X: f32 = 75.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;
```

This could also be done by using an external config file. This is
especially useful when you want to edit values a lot. Here, we're
keeping it simple.

## Create our next Component: The ball Component!

In `pong.rs`, let's create the `Ball` Component.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::prelude::{Component, DenseVecStorage};
pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}
```

A ball has a velocity and a radius, so we store that information in the component.

Then let's add a `initialise_ball` function the same way we wrote the
`initialise_paddles` function.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::assets::{Loader, AssetStorage};
# use amethyst::renderer::{Texture, PngFormat, TextureHandle, SpriteRender,
#                          TextureCoordinates, Sprite, SpriteSheet, SpriteSheetHandle, TextureMetadata};
# use amethyst::ecs::World;
# use amethyst::core::transform::Transform;
# use amethyst::ecs::prelude::{Component, DenseVecStorage};
# pub struct Ball {
#    pub velocity: [f32; 2],
#    pub radius: f32,
# }
# impl Component for Ball {
#    type Storage = DenseVecStorage<Self>;
# }
# const PADDLE_HEIGHT: f32 = 16.0;
# const PADDLE_WIDTH: f32 = 4.0;
# const SPRITESHEET_SIZE: (f32, f32) = (8.0, 16.0);
# const BALL_RADIUS: f32 = 2.0;
# const BALL_VELOCITY_X: f32 = 75.0;
# const BALL_VELOCITY_Y: f32 = 50.0;
# const ARENA_HEIGHT: f32 = 100.0;
# const ARENA_WIDTH: f32 = 100.0;
/// Initialises one ball in the middle-ish of the arena.
fn initialise_ball(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    // Create the translation.
    let mut local_transform = Transform::default();
    local_transform.set_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    // Assign the sprite for the ball
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // ball is the second sprite on the sprite sheet
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        })
        .with(local_transform)
        .build();
}
```

In [a previous chapter][pong_02_drawing] we saw how to load a sprite sheet
and get things drawn on the screen. Remember sprite sheet information
is stored in `pong_spritesheet.ron`, and the ball sprite was the
second one, whose index is `1`.

Finally, let's make sure the code is working as intended by updating the `on_start` method:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::renderer::{TextureHandle, SpriteSheetHandle};
# use amethyst::ecs::World;
# struct Paddle;
# impl amethyst::ecs::Component for Paddle {
#   type Storage = amethyst::ecs::VecStorage<Self>;
# }
# struct Ball;
# impl amethyst::ecs::Component for Ball {
#   type Storage = amethyst::ecs::VecStorage<Self>;
# }
# fn initialise_ball(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) { }
# fn initialise_paddles(world: &mut World, spritesheet: SpriteSheetHandle) { }
# fn initialise_camera(world: &mut World) { }
# fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle { unimplemented!() }
# struct MyState;
# impl SimpleState for MyState {
fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
    let world = data.world;

    // Load the spritesheet necessary to render the graphics.
    let sprite_sheet_handle = load_sprite_sheet(world);

    world.register::<Ball>(); // <- add this line temporarily

    initialise_ball(world, sprite_sheet_handle.clone()); // <- add this line
    initialise_paddles(world, sprite_sheet_handle);
    initialise_camera(world);
}
# }
```

Don't forget to call `clone` on `sprite_sheet_handle` because `initialise_paddles` and
`initialise_ball` *consume* the handle.

By running the game now, you should be able to see the two paddles and the ball
in the center. In the next section, we're going to make this ball actually move!

## Create systems to make the ball move

We're now ready to implement the `MoveBallsSystem` in `systems/move_balls.rs`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::prelude::{Component, DenseVecStorage};
#
# mod pong {
#     use amethyst::ecs::prelude::*;
#
#     pub struct Ball {
#        pub velocity: [f32; 2],
#        pub radius: f32,
#     }
#     impl Component for Ball {
#        type Storage = DenseVecStorage<Self>;
#     }
# }
#
use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    ecs::prelude::{Join, Read, ReadStorage, System, WriteStorage},
};

use crate::pong::Ball;

pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (balls, mut locals, time): Self::SystemData) {
        // Move every ball according to its speed, and the time passed.
        for (ball, local) in (&balls, &mut locals).join() {
            local.translate_x(ball.velocity[0] * time.delta_seconds());
            local.translate_y(ball.velocity[1] * time.delta_seconds());
        }
    }
}
#
# fn main() {}
```

This system is responsible for moving all balls according to their speed and
the elapsed time. Notice how the `join()` method is used to iterate over all
ball entities. Here we only have one ball, but if we ever need multiple, the
system will handle them out of the box.
In this system, we also want *framerate independence*.
That is, no matter the framerate, all objects move with the same speed.
To achieve that, a `delta time` since last frame is used.
This is commonly known as [`delta timing`][delta_timing].
As you can see in the snippet, to gain access to time passed since the last frame,
you need to use [`amethyst::core::timing::Time`][doc_time], a commonly used
resource. It has a method called `delta_seconds` that does exactly what we want.

Now that our ball can move, let's implement a new System:
`BounceSystem` in `systems/bounce.rs`.
It will be responsible for detecting collisions between balls and
paddles, as well as balls and the top and bottom edges of the arena.
If a collision is detected, the ball bounces off. This is done
by negating the velocity of the `Ball` component on the `x` or `y` axis.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::prelude::{Component, DenseVecStorage};
#
# mod pong {
#     use amethyst::ecs::prelude::*;
#
#     pub struct Ball {
#        pub velocity: [f32; 2],
#        pub radius: f32,
#     }
#     impl Component for Ball {
#        type Storage = DenseVecStorage<Self>;
#     }
#
#     #[derive(PartialEq, Eq)]
#     pub enum Side {
#       Left,
#       Right,
#     }
#
#     pub struct Paddle {
#       pub side: Side,
#       pub width: f32,
#       pub height: f32,
#     }
#     impl Component for Paddle {
#       type Storage = VecStorage<Self>;
#     }
#
#     pub const ARENA_HEIGHT: f32 = 100.0;
# }
#
use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Join, ReadStorage, System, WriteStorage},
};

use crate::pong::{Ball, Side, Paddle, ARENA_HEIGHT};

pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, (mut balls, paddles, transforms): Self::SystemData) {
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0 {
                ball.velocity[1] = -ball.velocity[1];
            } else if ball_y <= ball.radius && ball.velocity[1] < 0.0 {
                ball.velocity[1] = -ball.velocity[1];
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - paddle.width * 0.5;
                let paddle_y = paddle_transform.translation().y - paddle.height * 0.5;

                // To determine whether the ball has collided with a paddle, we create a larger
                // rectangle around the current one, by subtracting the ball radius from the
                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                // is then within the paddle if its centre is within the larger wrapper
                // rectangle.
                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + paddle.width + ball.radius,
                    paddle_y + paddle.height + ball.radius,
                ) {
                    if paddle.side == Side::Left && ball.velocity[0] < 0.0 {
                        ball.velocity[0] = -ball.velocity[0];
                    } else if paddle.side == Side::Right && ball.velocity[0] > 0.0 {
                        ball.velocity[0] = -ball.velocity[0];
                    }
                }
            }
        }
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
#
# fn main() {}
```

The following image illustrates how collisions with paddles are checked.

![Collision explanotary drawing](../images/pong_tutorial/pong_paddle_collision.png)

Also, don't forget to add `mod move_balls` and `mod bounce` in `systems/mod.rs`
as well as adding our new systems to the game data:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::core::transform::TransformBundle;
# use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline,
#                        PosTex, RenderBundle, Stage};
# fn main() -> amethyst::Result<()> {
# let path = "./resources/display_config.ron";
# let config = DisplayConfig::load(&path);
# let pipe = Pipeline::build().with_stage(Stage::with_backbuffer()
#       .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
#       .with_pass(DrawFlat::<PosTex>::new()),
# );
# mod systems {
# use amethyst;
# pub struct PaddleSystem;
# impl<'a> amethyst::ecs::System<'a> for PaddleSystem {
# type SystemData = ();
# fn run(&mut self, _: Self::SystemData) { }
# }
# pub struct MoveBallsSystem;
# impl<'a> amethyst::ecs::System<'a> for MoveBallsSystem {
# type SystemData = ();
# fn run(&mut self, _: Self::SystemData) { }
# }
# pub struct BounceSystem;
# impl<'a> amethyst::ecs::System<'a> for BounceSystem {
# type SystemData = ();
# fn run(&mut self, _: Self::SystemData) { }
# }
# }
# let input_bundle = amethyst::input::InputBundle::<String, String>::new();
let game_data = GameDataBuilder::default()
#    .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
#    .with_bundle(TransformBundle::new())?
#    .with_bundle(input_bundle)?
#    .with(systems::PaddleSystem, "paddle_system", &["input_system"])
    // ...other systems...
    .with(systems::MoveBallsSystem, "ball_system", &[])
    .with(
        systems::BounceSystem,
        "collision_system",
        &["paddle_system", "ball_system"],
    );
# Ok(())
# }
```

You should now have a ball moving and bouncing off paddles and off the top
and bottom of the screen. However, you will quickly notice that if the ball
goes out of the screen on the right or the left, it never comes back
and the game is over...

## Summary

In this chapter, we finally added a ball to our game. As always, the full code
is available under the `pong_tutorial_04` example in the Amethyst repository.
In the next chapter, we'll add a system checking when a player loses the game,
and add a scoring system!

[pong_02_drawing]: pong-tutorial-02.html#drawing
[doc_time]: https://www.amethyst.rs/doc/latest/doc/amethyst_core/timing/struct.Time.html
[delta_timing]: https://en.wikipedia.org/wiki/Delta_timing

