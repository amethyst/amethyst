use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::ecs::{Fetch, Join, ReadStorage, System, WriteStorage};
use amethyst::input::InputHandler;
use config::ArenaConfig;
use Paddle;
/// This system is responsible for moving all the paddles according to the user
/// provided input.
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        ReadStorage<'s, Paddle>,
        WriteStorage<'s, Transform>,
        Fetch<'s, Time>,
        Fetch<'s, InputHandler<String, String>>,
        Fetch<'s, ArenaConfig>,
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
                transform.translation[1] +=
                    paddle.velocity * time.delta_seconds() * movement as f32;

                // We make sure the paddle remains in the arena.
                transform.translation[1] = transform.translation[1]
                    .max(0.0)
                    .min(arena_config.height - paddle.height);
            }
        }
    }
}
