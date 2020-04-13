use amethyst::{
    core::{timing::Time, transform::Transform},
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};

use crate::{
    components::{Paddle, Side},
    ARENA_HEIGHT,
};

/// This system is responsible for moving all the paddles according to the user
/// provided input.
#[derive(SystemDesc)]
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        ReadStorage<'s, Paddle>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (paddles, mut transforms, time, input): Self::SystemData) {
        // Iterate over all paddles and move them according to the input the user
        // provided.
        for (paddle, transform) in (&paddles, &mut transforms).join() {
            let opt_movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };

            if let Some(movement) = opt_movement {
                // Update paddle position only when necessary
                if movement != 0.0 {
                    let paddle_y = transform.translation().y;
                    let scaled_amount = paddle.velocity * time.delta_seconds() * movement as f32;

                    transform.set_translation_y(
                        (paddle_y + scaled_amount)
                            // We make sure the paddle remains in the arena.
                            .max(paddle.height / 2.0)
                            .min(ARENA_HEIGHT - paddle.height / 2.0),
                    );
                }
            }
        }
    }
}
