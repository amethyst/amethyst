use {Ball, ScoreBoard};
use amethyst::assets::AssetStorage;
use amethyst::audio::Source;
use amethyst::audio::output::Output;
use amethyst::core::transform::LocalTransform;
use amethyst::ecs::{Fetch, FetchMut, Join, System, WriteStorage};
use audio::Sounds;

/// This system is responsible for checking if a ball has moved into a left or
/// a right edge. Points are distributed to the player on the other side, and
/// the ball is reset.
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, LocalTransform>,
        FetchMut<'s, ScoreBoard>,
        Fetch<'s, AssetStorage<Source>>,
        Fetch<'s, Sounds>,
        Fetch<'s, Option<Output>>,
    );

    fn run(
        &mut self,
        (mut balls, mut transforms, mut score_board, storage, sounds, audio_output):
        Self::SystemData,
){
        for (ball, transform) in (&mut balls, &mut transforms).join() {
            use ARENA_WIDTH;

            let ball_x = transform.translation[0];

            let did_hit = if ball_x <= ball.radius {
                // Right player scored on the left side.
                score_board.score_right += 1;
                true
            } else if ball_x >= ARENA_WIDTH - ball.radius {
                // Left player scored on the right side.
                score_board.score_left += 1;
                true
            } else {
                false
            };

            if did_hit {
                // Reset the ball.
                ball.velocity[0] = -ball.velocity[0];
                transform.translation[0] = ARENA_WIDTH / 2.0;

                // Print the score board.
                println!(
                    "Score: | {:^3} | {:^3} |",
                    score_board.score_left,
                    score_board.score_right
                );

                // Play audio.
                if let Some(ref output) = *audio_output {
                    if let Some(sound) = storage.get(&sounds.score_sfx) {
                        output.play_once(sound, 1.0);
                    }
                }
            }
        }
    }
}
