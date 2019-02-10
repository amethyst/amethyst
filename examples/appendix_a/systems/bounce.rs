use crate::{
    audio::{play_bounce, Sounds},
    config::ArenaConfig,
    Ball, Paddle, Side,
};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::transform::Transform,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage},
};

/// This system is responsible for detecting collisions between balls and
/// paddles, as well as balls and the top and bottom edges of the arena.
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Read<'s, Option<Output>>,
        Read<'s, ArenaConfig>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut balls, paddles, transforms, storage, sounds, audio_output, arena_config) = data;
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if ball_y <= ball.radius && ball.velocity[1] < 0.0 {
                ball.velocity[1] = -ball.velocity[1];
                play_bounce(&*sounds, &storage, &*audio_output);
            } else if ball_y >= arena_config.height - ball.radius && ball.velocity[1] > 0.0 {
                ball.velocity[1] = -ball.velocity[1];
                play_bounce(&*sounds, &storage, &*audio_output);
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x;
                let paddle_y = paddle_transform.translation().y;

                // To determine whether the ball has collided with a paddle, we create a larger
                // rectangle around the current one, by subtracting the ball radius from the
                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                // is than within the paddle if its centre is within the larger wrapper
                // rectangle.
                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + paddle.width + ball.radius,
                    paddle_y + paddle.height + ball.radius,
                ) {
                    if paddle.side == Side::Left && ball.velocity[0] < 0.0 {
                        ball.velocity[0] = -ball.velocity[0];
                        play_bounce(&*sounds, &storage, &*audio_output);
                    } else if paddle.side == Side::Right && ball.velocity[0] > 0.0 {
                        ball.velocity[0] = -ball.velocity[0];
                        play_bounce(&*sounds, &storage, &*audio_output);
                    }
                }
            }
        }
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right, but larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
