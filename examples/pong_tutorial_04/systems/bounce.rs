use amethyst::{
    core::transform::Transform,
    ecs::{Runnable, SystemBuilder},
    prelude::*,
};

use crate::pong::{Ball, Paddle, Side, ARENA_HEIGHT};

pub fn build() -> impl Runnable {
    SystemBuilder::new("BounceSystem")
        .with_query(<(&mut Ball, &Transform)>::query())
        // .with_query(<(&Entity, &mut Ball)>::query())
        .with_query(<(&Paddle, &Transform)>::query())
        .read_component::<Paddle>()
        .read_component::<Transform>()
        .write_component::<Ball>()
        .build(
            move |_commands, world, _resources, (query_balls, query_paddles)| {
                // Get the coordinates for each paddle. It is done outside of `query_balls` loop, as it borrows `world`
                // as immutable while `query_balls` borrows it as mutable
                let paddles = query_paddles
                    .iter(world)
                    .map(|(paddle, paddle_transform)| {
                        let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                        let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);
                        (
                            paddle_x,
                            paddle_y,
                            paddle.width,
                            paddle.height,
                            paddle.side.clone(),
                        )
                    })
                    .collect::<Vec<_>>();

                // Check whether a ball collided, and bounce off accordingly.
                //
                // We also check for the velocity of the ball every time, to prevent multiple collisions
                // from occurring.
                for (ball, transform) in query_balls.iter_mut(world) {
                    let ball_x = transform.translation().x;
                    let ball_y = transform.translation().y;

                    // Bounce at the top or the bottom of the arena.
                    if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                        || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
                    {
                        ball.velocity[1] = -ball.velocity[1];
                    }

                    // // Bounce at the paddles.
                    for (paddle_x, paddle_y, paddle_width, paddle_height, paddle_side) in
                        paddles.iter()
                    {
                        // To determine whether the ball has collided with a paddle, we create a larger
                        // rectangle around the current one, by subtracting the ball radius from the
                        // lowest coordinates, and adding the ball radius to the highest ones. The ball
                        // is then within the paddle if its centre is within the larger wrapper
                        // rectangle.
                        if point_in_rect(
                            ball_x,
                            ball_y,
                            paddle_x - ball.radius,
                            paddle_y - ball.radius,
                            paddle_x + (paddle_width + ball.radius),
                            paddle_y + (paddle_height + ball.radius),
                        ) && ((*paddle_side == Side::Left && ball.velocity[0] < 0.0)
                            || (*paddle_side == Side::Right && ball.velocity[0] > 0.0))
                        {
                            ball.velocity[0] = -ball.velocity[0];
                        }
                    }
                }
            },
        )
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
