use {Ball, Paddle};
use audio::Sounds;
use amethyst::audio::output::Output;
use amethyst::ecs::{Fetch, Join, ReadStorage, System, WriteStorage};
use amethyst::ecs::transform::LocalTransform;

/// This system is responsible for detecing collisions between balls and
/// paddles, as well as balls and the top and bottom edges of the arena.
pub struct CollisionSystem;

impl<'s> System<'s> for CollisionSystem {
    type SystemData = (WriteStorage<'s, Ball>,
     ReadStorage<'s, Paddle>,
     ReadStorage<'s, LocalTransform>,
     Fetch<'s, Sounds>,
     Fetch<'s, Option<Output>>);

    fn run(&mut self, (mut balls, paddles, transforms, sounds, audio_output): Self::SystemData) {
        // Check whether a ball collided, and bounce off accordingly.
        for (ball, transform) in (&mut balls, &transforms).join() {
            use ARENA_HEIGHT;

            let ball_x = transform.translation[0];
            let ball_y = transform.translation[1];

            // Bounce at the top of the bottom of the arena.
            if ball_y <= ball.radius || ball_y >= ARENA_HEIGHT - ball.radius {
                ball.velocity[1] = -ball.velocity[1];
                play_bounce(&*sounds, &*audio_output);
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation[0];
                let paddle_y = paddle_transform.translation[1];

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
                )
                {
                    ball.velocity[0] = -ball.velocity[0];
                    play_bounce(&*sounds, &*audio_output);
                }
            }
        }
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right, but
// larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}

/// Plays the bounce sound when a ball hits a side or a paddle.
fn play_bounce(sounds: &Sounds, audio_output: &Option<Output>) {
    use amethyst::audio::play::play_once;

    if let Some(ref audio_output) = *audio_output {
        play_once(&sounds.bounce_sfx, 1.0, &audio_output);
    }
}
