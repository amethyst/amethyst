use Ball;
use amethyst::ecs::{Fetch, Join, System, WriteStorage};
use amethyst::ecs::transform::LocalTransform;
use amethyst::timing::Time;

/// This system is responsible for moving all balls according to their speed
/// and the time passed.
pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, LocalTransform>,
        Fetch<'s, Time>,
    );

    fn run(&mut self, (mut balls, mut locals, time): Self::SystemData) {
        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        // Move every ball according to its speed, and the time passed.
        for (ball, local) in (&mut balls, &mut locals).join() {
            local.translation[0] += ball.velocity[0] * delta_time;
            local.translation[1] += ball.velocity[1] * delta_time;
        }
    }
}
