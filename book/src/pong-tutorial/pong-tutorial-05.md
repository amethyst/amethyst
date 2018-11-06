# Winning Rounds and Keeping Score

Our last chapter ended on a bit of a cliffhanger. What happens when our ball
reaches the left or right edge of the screen? It just keeps going! 😦

In this chapter, we'll fix that by putting the ball back into play after it
leaves either side of the arena. We'll also add a scoreboard and keep track of
who's winning and losing.


## Winning and Losing Rounds

So let's fix the big current issue; having a game that only works for one
round isn't very fun. We'll add a new system that will check if the ball has
reached either edge of the arena and reset its position and velocity. We'll also
make a note of who got the point for the round.

First, we'll add a new module to `systems/mod.rs`
```rust,no_run,noplaypen,ignore
mod winner;

pub use self::winner::WinnerSystem;
```

Then, we'll create `systems/winner.rs`:

```rust,no_run,noplaypen
# extern crate amethyst;
#
# mod pong {
#     use amethyst::ecs::prelude::*;
#
#     pub struct Ball {
#         pub radius: f32,
#         pub velocity: [f32; 2],
#     }
#     impl Component for Ball {
#         type Storage = DenseVecStorage<Self>;
#     }
#
#     pub const ARENA_WIDTH: f32 = 100.0;
# }
#
use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Join, System, WriteStorage},
};

use pong::{Ball, ARENA_WIDTH};

pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (mut balls, mut locals): Self::SystemData) {
        for (ball, transform) in (&mut balls, &mut locals).join() {
            let ball_x = transform.translation[0];

            let did_hit = if ball_x <= ball.radius {
                // Right player scored on the left side.
                println!("Player 2 Scores!");
                true
            } else if ball_x >= ARENA_WIDTH - ball.radius {
                // Left player scored on the right side.
                println!("Player 1 Scores!");
                true
            } else {
                false
            };

            if did_hit {
                ball.velocity[0] = -ball.velocity[0]; // Reverse Direction
                transform.translation[0] = ARENA_WIDTH / 2.0; // Reset Position
            }
        }
    }
}
#
# fn main() {}
```

Here, we're creating a new system, joining on all `Entities` that have a `Ball`
and a `Transform` component, and then checking each ball to see if it has
reached either the left or right boundary of the arena. If so, then we reverse
its direction and put it back in the middle of the screen.

Now, we just need to add our new system to `main.rs`, and we should be able to
keep playing after someone scores, and log who got the point.

```rust,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#    prelude::*,
#    core::transform::TransformBundle,
#    renderer::{
#        DisplayConfig,
#        DrawSprite,
#        Pipeline,
#        RenderBundle,
#        Stage,
#    }
# };
#
# mod systems {
#     use amethyst;
#     pub struct PaddleSystem;
#     impl<'a> amethyst::ecs::System<'a> for PaddleSystem {
#         type SystemData = ();
#         fn run(&mut self, _: Self::SystemData) { }
#     }
#     pub struct MoveBallsSystem;
#     impl<'a> amethyst::ecs::System<'a> for MoveBallsSystem {
#         type SystemData = ();
#         fn run(&mut self, _: Self::SystemData) { }
#     }
#     pub struct BounceSystem;
#     impl<'a> amethyst::ecs::System<'a> for BounceSystem {
#         type SystemData = ();
#         fn run(&mut self, _: Self::SystemData) { }
#     }
#     pub struct WinnerSystem;
#     impl<'a> amethyst::ecs::System<'a> for WinnerSystem {
#         type SystemData = ();
#         fn run(&mut self, _: Self::SystemData) { }
#     }
# }
#
# fn main() -> amethyst::Result<()> {
#
# let path = "./resources/display_config.ron";
# let config = DisplayConfig::load(&path);
# let pipe = Pipeline::build().with_stage(Stage::with_backbuffer()
#       .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
#       .with_pass(DrawSprite::new()),
# );
# let input_bundle = amethyst::input::InputBundle::<String, String>::new();
#
let game_data = GameDataBuilder::default()
#    .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
#    .with_bundle(TransformBundle::new())?
#    .with_bundle(input_bundle)?
#    .with(systems::PaddleSystem, "paddle_system", &["input_system"])
#    .with(systems::MoveBallsSystem, "ball_system", &[])
#    .with(
#        systems::BounceSystem,
#        "collision_system",
#        &["paddle_system", "ball_system"],
#    )
    // ...other systems...
    .with(systems::WinnerSystem, "winner_system", &["ball_system"]);
#
# Ok(())
# }
```


## Adding a Scoreboard

[//]: # "TODO: Setup UiBundle in main.rs"
[//]: # "TODO: Initialize scoreboard in pong.rs"


## Updating the Scoreboard

[//]: # "TODO: Update player scores in WinnerSystem"


## Summary

[//]: # "TODO: Go over main additions"
[//]: # "TODO: Introduce next chapter (likely music/audio?)"
