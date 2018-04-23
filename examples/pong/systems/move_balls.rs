use Ball;
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::ecs::prelude::{Join, Read, System, WriteStorage};

/// This system is responsible for moving all balls according to their speed
/// and the time passed.
pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut balls, mut locals, time): Self::SystemData) {
        // Move every ball according to its speed, and the time passed.
        for (ball, local) in (&mut balls, &mut locals).join() {
            local.translation[0] += ball.velocity[0] * time.delta_seconds();
            local.translation[1] += ball.velocity[1] * time.delta_seconds();
        }
    }
}
