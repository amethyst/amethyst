use crate::{
    audio::{play_sound, Sounds},
    Ball, Paddle, Side, ARENA_HEIGHT, BALL_RADIUS,
};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
};

const BOUNDARY_TOP: f32 = ARENA_HEIGHT - BALL_RADIUS;
const BOUNDARY_BOTTOM: f32 = BALL_RADIUS;

/// This system is responsible for detecting collisions between balls and
/// paddles, as well as balls and the top and bottom edges of the arena.
#[derive(SystemDesc)]
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
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
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if (ball_y <= BOUNDARY_BOTTOM && ball.heads_down())
                || (ball_y >= BOUNDARY_TOP && ball.heads_up())
            {
                ball.reverse_y();
                play_sound(&sounds.bounce, &storage, audio_output.as_deref());
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width / 2.0);
                let paddle_y = paddle_transform.translation().y - (paddle.height / 2.0);

                if point_in_rect(ball_x, ball_y, paddle_x, paddle_y)
                    && ((paddle.side == Side::Left && ball.heads_left())
                        || (paddle.side == Side::Right && ball.heads_right()))
                {
                    ball.reverse_x();
                    play_sound(&sounds.bounce, &storage, audio_output.as_deref());
                }
            }
        }
    }
}

// To determine whether the ball has collided with a paddle, we create a larger
// rectangle around the current one, by subtracting the ball radius from the
// lowest coordinates, and adding the ball radius to the highest ones. The ball
// is then within the paddle if its centre is within the larger wrapper
// rectangle.
fn point_in_rect(ball_x: f32, ball_y: f32, paddle_x: f32, paddle_y: f32) -> bool {
    let left = paddle_x - BALL_RADIUS;
    let bottom = paddle_y - BALL_RADIUS;
    let right = paddle_x + PADDLE_WIDTH + BALL_RADIUS;
    let top = paddle_y + PADDLE_HEIGHT + BALL_RADIUS;

    // A point is in a box when its coordinates are smaller or equal than the top
    // right and larger or equal than the bottom left.
    (ball_x >= left) && (ball_y >= bottom) && (ball_x <= right) && (ball_y <= top)
}
