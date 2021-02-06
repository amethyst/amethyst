use amethyst::{core::Transform, ecs::SystemBuilder, prelude::*};

use crate::pong::{Ball, Paddle, Side, ARENA_HEIGHT};

pub struct BounceSystem;

impl System for BounceSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PaddleSystem")
                .read_component::<Paddle>()
                .read_component::<Transform>()
                .write_component::<Ball>()
                .with_query(<(&mut Ball, &Transform)>::query())
                .with_query(<(&Paddle, &Transform)>::query())
                .build(|_, world, _, (ball_query, paddle_query)| {
                    let (mut balls, paddles) = world.split_for_query(ball_query);
                    // Check whether a ball collided, and bounce off accordingly.
                    //
                    // We also check for the velocity of the ball every time, to prevent multiple collisions
                    // from occurring.
                    for (ball, transform) in ball_query.iter_mut(&mut balls) {
                        let ball_x = transform.translation().x;
                        let ball_y = transform.translation().y;

                        // Bounce at the top or the bottom of the arena.
                        if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                            || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
                        {
                            ball.velocity[1] = -ball.velocity[1];
                        }

                        // Bounce at the paddles.
                        for (paddle, paddle_transform) in paddle_query.iter(&paddles) {
                            let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                            let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);

                            // To determine whether the ball has collided with a paddle, we create a larger
                            // rectangle around the current one, by subtracting the ball radius from the
                            // lowest coordinates, and adding the ball radius to the highest ones. The ball
                            // is then within the paddle if its center is within the larger wrapper
                            // rectangle.
                            if point_in_rect(
                                ball_x,
                                ball_y,
                                paddle_x - ball.radius,
                                paddle_y - ball.radius,
                                paddle_x + paddle.width + ball.radius,
                                paddle_y + paddle.height + ball.radius,
                            ) {
                                if (paddle.side == Side::Left && ball.velocity[0] < 0.0)
                                    || (paddle.side == Side::Right && ball.velocity[0] > 0.0)
                                {
                                    ball.velocity[0] = -ball.velocity[0];
                                }
                            }
                        }
                    }

                }),
        )
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
