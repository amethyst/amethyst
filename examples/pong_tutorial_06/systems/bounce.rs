use amethyst::{
    assets::AssetStorage,
    audio::{output::OutputWrapper, Source},
    core::transform::Transform,
    ecs::SystemBuilder,
    prelude::*,
};

use crate::{
    audio::{play_bounce, Sounds},
    pong::{Ball, Paddle, Side, ARENA_HEIGHT},
};

pub struct BounceSystem;

impl System for BounceSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("BounceSystem")
                .read_resource::<Sounds>()
                .read_resource::<AssetStorage<Source>>()
                .read_resource::<OutputWrapper>()
                .with_query(<(&mut Ball, &Transform)>::query())
                .with_query(<&Paddle>::query())
                .read_component::<Paddle>()
                .read_component::<Transform>()
                .write_component::<Ball>()
                .build(
                    move |_commands,
                          world,
                          (sounds, storage, output_wrapper),
                          (query_balls, query_paddles)| {
                        let (mut ball_world, remaining) = world.split_for_query(query_balls);

                        // Check whether a ball collided, and bounce off accordingly.
                        //
                        // We also check for the velocity of the ball every time, to prevent multiple collisions
                        // from occurring.
                        for (ball, transform) in query_balls.iter_mut(&mut ball_world) {
                            let ball_x = transform.translation().x;
                            let ball_y = transform.translation().y;

                            // Bounce at the top or the bottom of the arena.
                            if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                                || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
                            {
                                ball.velocity[1] = -ball.velocity[1];
                            }

                            // Bounce at the paddles.
                            for paddle in query_paddles.iter(&remaining) {
                                // To determine whether the ball has collided with a paddle, we create a larger
                                // rectangle around the current one, by subtracting the ball radius from the
                                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                                // is then within the paddle if its centre is within the larger wrapper
                                // rectangle.
                                if point_in_rect(
                                    ball_x,
                                    ball_y,
                                    paddle.x - paddle.width / 2. - ball.radius,
                                    paddle.y - paddle.height / 2. - ball.radius,
                                    paddle.x + paddle.width / 2. + ball.radius,
                                    paddle.y + paddle.height / 2. + ball.radius,
                                ) && ((paddle.side == Side::Left && ball.velocity[0] < 0.0)
                                    || (paddle.side == Side::Right && ball.velocity[0] > 0.0))
                                {
                                    println!("Bounce!");
                                    play_bounce(sounds, storage, output_wrapper.output.as_ref());
                                    ball.velocity[0] = -ball.velocity[0];
                                }
                            }
                        }
                    },
                ),
        )
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
