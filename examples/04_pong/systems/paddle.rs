use Paddle;
use amethyst::ecs::{Fetch, Join, System, WriteStorage};
use amethyst::ecs::transform::LocalTransform;
use amethyst::input::InputHandler;
use amethyst::timing::Time;

/// This system is responsible for moving all the paddles according to the user
/// provided input.
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        WriteStorage<'s, Paddle>,
        WriteStorage<'s, LocalTransform>,
        Fetch<'s, Time>,
        Fetch<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (mut paddles, mut transforms, time, input): Self::SystemData) {
        use Side;

        // Because the time between frames is not always constant, we use the time
        // difference between this frame and the previous one to make sure the movement
        // speed of the paddles does remain stable.
        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        // Iterate over all planks and move them according to the input the user
        // provided.
        for (paddle, transform) in (&mut paddles, &mut transforms).join() {
            let opt_movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };

            if let Some(movement) = opt_movement {
                use ARENA_HEIGHT;
                transform.translation[1] += paddle.velocity * delta_time * movement as f32;

                // We make sure the paddle remains in the arena.
                transform.translation[1] = transform.translation[1]
                    .max(0.0)
                    .min(ARENA_HEIGHT - paddle.height);
            }
        }
    }
}
