use amethyst::core::transform::components::Transform;
use amethyst::core::cgmath::num_traits::clamp;
use amethyst::ecs::prelude::{Join, Read, ReadStorage, System, WriteStorage};
use amethyst::input::InputHandler;
use pong::{Paddle, Side, PADDLE_HEIGHT};

pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Paddle>,
        Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (mut transforms, paddles, input): Self::SystemData) {
        for (paddle, mut transform) in (&paddles, &mut transforms).join() {
            let movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };
            if let Some(mv_amount) = movement {
                let scaled_amount = (1.0 / 60.0) * mv_amount as f32;
                let mut position = transform.position();
                position[1] = clamp(position[1] + scaled_amount, 1.0 - PADDLE_HEIGHT, -1.0);
            }
        }
    }
}
