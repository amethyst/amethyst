use Paddle;
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::core::cgmath::Vector3;
use amethyst::ecs::prelude::{Join, Read, ReadStorage, System, WriteStorage};
use amethyst::input::InputHandler;
use config::ArenaConfig;
use amethyst::core::cgmath::num_traits::clamp;
/// This system is responsible for moving all the paddles according to the user
/// provided input.
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        ReadStorage<'s, Paddle>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, InputHandler<String, String>>,
        Read<'s, ArenaConfig>,
    );

    fn run(&mut self, (paddles, mut transforms, time, input, arena_config): Self::SystemData) {
        use Side;

        // Iterate over all planks and move them according to the input the user
        // provided.
        for (paddle, transform) in (&paddles, &mut transforms).join() {
            let opt_movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };

            if let Some(movement) = opt_movement {
                let calc_movement = clamp(
                    paddle.velocity * time.delta_seconds() * movement as f32,
                    0.0,
                    arena_config.height - paddle.height
                );
                transform.set_position(Vector3 { x: 0., y: calc_movement, z: 0. });
            }
        }
    }
}
