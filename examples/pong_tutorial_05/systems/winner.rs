use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::{Runnable, SystemBuilder},
    ui::UiText,
};

use crate::pong::{Ball, ScoreBoard, ScoreText, ARENA_WIDTH};

pub fn build() -> impl Runnable {
    SystemBuilder::new("WinnerSystem")
        .with_query(<(&Ball, &mut Transform)>::query())
        .with_query(Write::<UiText>::query())
        .write_component::<Ball>()
        .write_component::<Transform>()
        .write_component::<UiText>()
        .write_resource::<ScoreBoard>()
        .read_resource::<ScoreText>()
        .build(
            move |_commands, world, (score_board, score_text), (balls_query, edit_query)| {
                for (ball, transform) in balls_query.iter_mut(world) {
                    let ball_x = transform.translation().x;
                    let did_hit = if ball_x <= ball.radius {
                        // Right player scored on the left side.
                        // We top the score at 999 to avoid text overlap.
                        score_board.score_right = (score_board.score_right + 1).min(999);
                        if let Some(text) = edit_query.get_mut(score_text.p2_score) {
                            text.text = score_board.score_right.to_string();
                        }
                        true
                    } else if ball_x >= ARENA_WIDTH - ball.radius {
                        // Left player scored on the right side.
                        // We top the score at 999 to avoid text overlap.
                        score_board.score_left = (score_board.score_left + 1).min(999);
                        if let Some(text) = edit_query.get_mut(score_text.p1_score) {
                            text.text = score_board.score_left.to_string();
                        }
                        true
                    } else {
                        false
                    };
                    if did_hit {
                        // Reset the ball.
                        ball.velocity[0] = -ball.velocity[0];
                        transform.set_translation_x(ARENA_WIDTH / 2.0);
                        // Print the score board.
                        println!(
                            "Score: | {:^3} | {:^3} |",
                            score_board.score_left, score_board.score_right
                        );
                    }
                }
            },
        )
}
