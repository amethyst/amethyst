use amethyst::{
    core::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, ReadExpect, System, SystemData, Write, WriteStorage},
    ui::UiText,
};

use crate::{
    components::Ball,
    pong::{ScoreBoard, ScoreText},
    ARENA_HEIGHT, ARENA_WIDTH, BALL_RADIUS,
};

const BALL_BOUNDARY_RIGHT: f32 = ARENA_HEIGHT - BALL_RADIUS;
const BALL_BOUNDARY_LEFT: f32 = BALL_RADIUS;

/// This system is responsible for checking if a ball has moved into a left or
/// a right edge. Points are distributed to the player on the other side, and
/// the ball is reset.
#[derive(SystemDesc)]
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, ScoreBoard>,
        ReadExpect<'s, ScoreText>,
    );

    fn run(
        &mut self,
        (mut balls, mut transforms, mut text, mut score_board, score_text): Self::SystemData,
    ) {
        for (ball, transform) in (&mut balls, &mut transforms).join() {
            let ball_x = transform.translation().x;

            let scored = ball_x <= BALL_BOUNDARY_LEFT || ball_x >= BALL_BOUNDARY_RIGHT;

            if ball_x <= BALL_BOUNDARY_LEFT {
                // Right player scored on the left side.
                // We top the score at 999 to avoid text overlap.
                score_board.score_right = (score_board.score_right + 1).min(999);
                if let Some(text) = text.get_mut(score_text.p2_score) {
                    text.text = score_board.score_right.to_string();
                }
            } else if ball_x >= BALL_BOUNDARY_RIGHT {
                // Left player scored on the right side.
                // We top the score at 999 to avoid text overlap.
                score_board.score_left = (score_board.score_left + 1).min(999);
                if let Some(text) = text.get_mut(score_text.p1_score) {
                    text.text = score_board.score_left.to_string();
                }
            }

            if scored {
                // Reset the ball.
                ball.reverse_x();
                transform.set_translation_x(ARENA_WIDTH / 2.0);
                transform.set_translation_y(ARENA_HEIGHT / 2.0);
            }
        }
    }
}
