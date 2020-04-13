use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};

use crate::{
    components::{Paddle, Side},
    ARENA_HEIGHT,
};

#[derive(SystemDesc)]
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        ReadStorage<'s, Paddle>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (paddles, mut transforms, input): Self::SystemData) {
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
                    // Get current y position of the paddle
                    let paddle_y = transform.translation().y;
                    let scaled_amount = paddle.velocity * movement as f32;

                    transform.set_translation_y(
                        (paddle_y + scaled_amount)
                            .max(paddle.height / 2.0)
                            .min(ARENA_HEIGHT - paddle.height / 2.0),
                    );
                }
            }
        }
    }
}
