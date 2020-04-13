# Making a ball move and bounce

In the previous chapter, we learned how to capture user input
to make things move on the screen by creating a `System` ourselves.
This chapter will reuse all the knowledge we acquired through the
previous chapters to add a new object to our game: a ball that moves
and bounces around!

First, let's define some other useful constants for this chapter in `main.rs`:

```rust,edition2018,no_run,noplaypen
const BALL_VELOCITY_X: f32 = 75.0;
const BALL_VELOCITY_Y: f32 = 50.0;
const BALL_RADIUS: f32 = 2.0;
```

This could also be done by using an external config file. This is
especially useful when you want to edit values a lot. Here, we're
keeping it simple.

## Create our next Component: The ball Component!

Let's create the `Ball` component in `components/ball.rs`.

```rust,edition2018,no_run,noplaypen
use amethyst::ecs::{Component, DenseVecStorage};

use crate::{BALL_VELOCITY_X, BALL_VELOCITY_Y, BALL_RADIUS};

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

impl Ball {
    pub fn new() -> Ball {
        Ball {
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
            radius: BALL_RADIUS,
        }
    }

    pub fn reverse_x(&mut self) {
        self.velocity[0] = -self.velocity[0];
    }

    pub fn reverse_y(&mut self) {
        self.velocity[1] = -self.velocity[1];
    }

    pub fn heads_up(&self) -> bool {
        self.velocity[1] > 0.0
    }

    pub fn heads_down(&self) -> bool {
        self.velocity[1] < 0.0
    }

    pub fn heads_right(&self) -> bool {
        self.velocity[0] > 0.0
    }

    pub fn heads_left(&self) -> bool {
        self.velocity[0] < 0.0
    }
}
```

A ball has a velocity and a radius, so we store that information in the component.

Now let's update the `components/mod.rs`
```rust,edition2018,no_run,noplaypen
mod ball;
mod paddle;

pub use self::{
    ball::Ball,
    paddle::{Paddle, Side},
};
```

Then in `pong.rs` add an `initialise_ball` function the same way we wrote the
`initialise_paddles` function.

```rust,edition2018,no_run,noplaypen
use crate::{
    components::{/* ... */, Ball}, // Add Ball here
    /* ... */
};

/* ... */

/// Initialises one ball in the middle-ish of the arena.
fn initialise_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    // Create the translation.
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    // Assign the sprite for the ball
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // ball is the second sprite on the sprite sheet
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Ball::new())
        .with(transform)
        .build();
}
```

In [a previous chapter][pong_02_drawing] we saw how to load a sprite sheet
and get things drawn on the screen. Remember sprite sheet information
is stored in `pong_spritesheet.ron`, and the ball sprite was the
second one, whose index is `1`.

Finally, let's make sure the code is working as intended by updating the `on_start` method:

```rust,edition2018,no_run,noplaypen
# /* ... */
#
# struct MyState;
#
# impl SimpleState for MyState {
fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
    let world = data.world;

    // Load the spritesheet necessary to render the graphics.
    let sprite_sheet_handle = load_sprite_sheet(world);

    world.register::<Ball>(); // <- add this line temporarily

    initialise_camera(world);
    initialise_paddles(world, sprite_sheet_handle.clone()); // add .clone() here
    initialise_ball(world, sprite_sheet_handle); // <- add this line
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
use amethyst::{
    core::{timing::Time, transform::Transform},
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

use crate::components::Ball;

#[derive(SystemDesc)]
pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (balls, mut transforms, time): Self::SystemData) {
        // Move every ball according to its speed, and the time passed.
        for (ball, transform) in (&balls, &mut transforms).join() {
            transform.prepend_translation_x(ball.velocity[0] * time.delta_seconds());
            transform.prepend_translation_y(ball.velocity[1] * time.delta_seconds());
        }
    }
}
```
`MoveBallsSystem` is responsible for moving all balls according to their speed and
the elapsed time. Notice how the `join()` method is used to iterate over all
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

```rust,edition2018,no_run,noplaypen
use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, ReadStorage, System, SystemData, WriteStorage},
};

use crate::{
    components::{Ball, Paddle, Side},
    ARENA_HEIGHT, BALL_RADIUS, PADDLE_HEIGHT, PADDLE_WIDTH,
};

const BALL_BOUNDARY_TOP: f32 = ARENA_HEIGHT - BALL_RADIUS;
const BALL_BOUNDARY_BOTTOM: f32 = BALL_RADIUS;

/// This system is responsible for detecting collisions between balls and
/// paddles, as well as balls and the top and bottom edges of the arena.
#[derive(SystemDesc)]
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
    );

    fn run(
        &mut self,
        (mut balls, paddles, transforms): Self::SystemData,
    ) {
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if (ball_y <= BALL_BOUNDARY_BOTTOM && ball.heads_down())
                || (ball_y >= BALL_BOUNDARY_TOP && ball.heads_up())
            {
                ball.reverse_y();
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width / 2.0);
                let paddle_y = paddle_transform.translation().y - (paddle.height / 2.0);

                if point_in_rect(ball_x, ball_y, paddle_x, paddle_y)
                    && ((paddle.side == Side::Left && ball.heads_left())
                        || (paddle.side == Side::Right && ball.heads_right()))
                {
                    ball.reverse_x();
                }
            }
        }
    }
}

// To determine whether the ball has collided with a paddle, we create a larger
// rectangle around the current one, by subtracting the ball radius from the
// lowest coordinates, and adding the ball radius to the highest ones. The ball
// is then within the paddle if its centre is within the larger wrapper
// rectangle.
fn point_in_rect(ball_x: f32, ball_y: f32, paddle_x: f32, paddle_y: f32) -> bool {
    let left = paddle_x - BALL_RADIUS;
    let bottom = paddle_y - BALL_RADIUS;
    let right = paddle_x + PADDLE_WIDTH + BALL_RADIUS;
    let top = paddle_y + PADDLE_HEIGHT + BALL_RADIUS;

    // A point is in a box when its coordinates are smaller or equal than the top
    // right and larger or equal than the bottom left.
    (ball_x >= left) && (ball_y >= bottom) && (ball_x <= right) && (ball_y <= top)
}
```
The following image illustrates how collisions with paddles are checked.

![Collision explanotary drawing](../images/pong_tutorial/pong_paddle_collision.png)

Don't forget to update `systems/mod.rs`:
```rust,edition2018,no_run,noplaypen
mod bounce;
mod move_balls;
mod paddle;

pub use self::bounce::BounceSystem;
pub use self::move_balls::MoveBallsSystem;
pub use self::paddle::PaddleSystem;
```
Now, let's add our new systems to the game data:

```rust,edition2018,no_run,noplaypen
fn main() -> amethyst::Result<()> {
/* ... */

let game_data = GameDataBuilder::default()
    /* ... */
#     .with_bundle(render_bundle)?
#     .with_bundle(TransformBundle::new())?
#     .with_bundle(input_bundle)?
#     .with(PaddleSystem, "paddle_system", &["input_system"]);
    .with(MoveBallsSystem, "ball_system", &[])
    .with(
        BounceSystem,
        "collision_system",
        &["paddle_system", "ball_system"],
    );
/* ... */
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

First, let's add a new `update` method to our state below the `on_start` in `pong.rs`.

```rust,edition2018,no_run,noplaypen
# struct Pong;
# impl SimpleState for Pong {
# fn on_start() { /* ... */ }
#
fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
    Trans::None
}
# }
```

That method allows you to transition out of state using its return value.
Here, we do not want to change any state, so we return `Trans::None`.

Now we have to move paddle creation to that method and add some delay to it. Our `update` runs every frame,
so in order to do something only once after a given time, we have to use our local state.
Additionally, notice that `initialise_paddles` requires us to provide the `sprite_sheet_handle`, but it was created
as a local variable inside `on_start`. For that reason, we have to make it a part of the state too.

Let's add some fields to our `Pong` struct:

```rust,edition2018,no_run,noplaypen
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

```rust,edition2018,no_run,noplaypen
# /* ... */
#
# fn main() -> amethyst::Result<()> {
# /* ... */
#
let mut game = Application::new(assets_dir, Pong::default(), game_data)?;
#
# /* ... */
# }
```

Now let's finish our timer and ball spawning code. We have to do two things:
- First, we have to initialize our state and remove `initialise_ball` from `on_start`,
- then we have to `initialise_ball` once after the time has passed inside `update`:

```rust,edition2018,no_run,noplaypen
use amethyst::{
    /* ... */
    core::{/* ... */, timing::Time}, // add timing:Time
};

# #[derive(Default)] pub struct Pong {
#     ball_spawn_timer: Option<f32>,
#     sprite_sheet_handle: Option<Handle<SpriteSheet>>,
# }
# 
impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Wait one second before spawning the ball.
        self.ball_spawn_timer.replace(1.0);

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        self.sprite_sheet_handle.replace(load_sprite_sheet(world));
        initialise_camera(world);
        initialise_paddles(world, self.sprite_sheet_handle.unwrap());
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(mut timer) = self.ball_spawn_timer.take() {
            // If the timer isn't expired yet, subtract the time that passed since the last update.
            {
                let time = data.world.fetch::<Time>();
                timer -= time.delta_seconds();
            }
            if timer <= 0.0 {
                // When timer expire, spawn the ball
                initialise_ball(data.world, self.sprite_sheet_handle.clone().unwrap());
            } else {
                // If timer is not expired yet, put it back onto the state.
                self.ball_spawn_timer.replace(timer);
            }
        }
        Trans::None
    }
}
#
# fn initialise_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) { /* ... */ }
# fn initialise_paddles(world: &mut World, spritesheet: Handle<SpriteSheet>) { /* ... */ }
# fn initialise_camera(world: &mut World) { /* ... */ }
# fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> { /* ... */ }
```

Now our ball will only show up after a set delay, giving us some breathing room after startup.
This will give us a better opportunity to see what happens to the ball immediately when it spawns.

## Summary

In this chapter, we finally added a ball to our game. As always, the full code
is available under the `pong_tutorial_04` example in the Amethyst repository.
In the next chapter, we'll add a system checking when a player loses the game,
and add a scoring system!

[pong_02_drawing]: pong-tutorial-02.html#drawing
[doc_time]: https://docs.amethyst.rs/stable/amethyst_core/timing/struct.Time.html
[delta_timing]: https://en.wikipedia.org/wiki/Delta_timing

