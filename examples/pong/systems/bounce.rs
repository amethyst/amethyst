use crate::{
    audio::{play_bounce, Sounds},
    BallComponent, PaddleComponent, Side,
};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{transform::TransformComponent, Float},
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage},
};
use std::ops::Deref;

/// This system is responsible for detecting collisions between balls and
/// paddles, as well as balls and the top and bottom edges of the arena.
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, BallComponent>,
        ReadStorage<'s, PaddleComponent>,
        ReadStorage<'s, TransformComponent>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (mut balls, paddles, transforms, storage, sounds, audio_output): Self::SystemData,
    ) {
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            use crate::ARENA_HEIGHT;

            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if (ball_y <= Float::from(ball.radius) && ball.velocity[1] < 0.0)
                || (ball_y >= Float::from(ARENA_HEIGHT - ball.radius) && ball.velocity[1] > 0.0)
            {
                ball.velocity[1] = -ball.velocity[1];
                play_bounce(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5).into();
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5).into();

                // To determine whether the ball has collided with a paddle, we create a larger
                // rectangle around the current one, by subtracting the ball radius from the
                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                // is then within the paddle if its centre is within the larger wrapper
                // rectangle.
                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius.into(),
                    paddle_y - ball.radius.into(),
                    paddle_x + (paddle.width + ball.radius).into(),
                    paddle_y + (paddle.height + ball.radius).into(),
                ) {
                    if (paddle.side == Side::Left && ball.velocity[0] < 0.0)
                        || (paddle.side == Side::Right && ball.velocity[0] > 0.0)
                    {
                        ball.velocity[0] = -ball.velocity[0];
                        play_bounce(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                    }
                }
            }
        }
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: Float, y: Float, left: Float, bottom: Float, right: Float, top: Float) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
