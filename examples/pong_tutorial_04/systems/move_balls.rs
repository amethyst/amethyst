use amethyst::{
    core::{Time, transform::Transform},
    ecs::SystemBuilder,
    prelude::*,
};

use crate::pong::Ball;

pub struct BallSystem;

impl System for BallSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MoveBallsSystem")
                .with_query(<(&Ball, &mut Transform)>::query())
                .read_resource::<Time>()
                .read_component::<Ball>()
                .write_component::<Transform>()
                .build(move |_commands, world, time, query_balls| {
                    for (ball, local) in query_balls.iter_mut(world) {
                        local.prepend_translation_x(ball.velocity[0] * time.delta_time().as_secs_f32());
                        local.prepend_translation_y(ball.velocity[1] * time.delta_time().as_secs_f32());
                    }
                }),
        )
    }
}
