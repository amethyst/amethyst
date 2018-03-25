use pong::{Paddle, Side, PADDLE_HEIGHT};
use amethyst::ecs::{Fetch, Join, System};
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
}
