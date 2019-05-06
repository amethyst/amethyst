use crate::Ball;
use amethyst::{
    core::{timing::Time, transform::Transform},
    ecs::prelude::{Join, Read, System, WriteStorage},
};

/// This system is responsible for moving all balls according to their speed
/// and the time passed.
pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform<f32>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut balls, mut locals, time): Self::SystemData) {
        // Move every ball according to its speed, and the time passed.
        for (ball, local) in (&mut balls, &mut locals).join() {
            local.prepend_translation_x(ball.velocity[0] * time.delta_seconds());
            local.prepend_translation_y(ball.velocity[1] * time.delta_seconds());
        }
    }
}
