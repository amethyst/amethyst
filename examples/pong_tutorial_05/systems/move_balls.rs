use amethyst::{
    core::{timing::Time, transform::TransformComponent},
    ecs::prelude::{Join, Read, ReadStorage, System, WriteStorage},
};

use crate::pong::BallComponent;

pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        ReadStorage<'s, BallComponent>,
        WriteStorage<'s, TransformComponent>,
        Read<'s, Time>,
    );

    fn run(&mut self, (balls, mut locals, time): Self::SystemData) {
        // Move every ball according to its speed, and the time passed.
        for (ball, local) in (&balls, &mut locals).join() {
            local.prepend_translation_x(ball.velocity[0] * time.delta_seconds());
            local.prepend_translation_y(ball.velocity[1] * time.delta_seconds());
        }
    }
}
