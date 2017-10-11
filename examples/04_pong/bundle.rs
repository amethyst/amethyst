use {Ball, Paddle, ScoreBoard};
use amethyst::Result;
use amethyst::ecs::ECSBundle;
use amethyst::prelude::*;
use amethyst::timing::Time;
use systems::{BounceSystem, MoveBallsSystem, PaddleSystem, WinnerSystem};

/// A bundle is a convenient way to initialise related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
pub struct PongBundle;

impl<'a, 'b, T> ECSBundle<'a, 'b, T> for PongBundle {
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        Ok(
            builder
                .with_resource(ScoreBoard::new())
                .with_resource(Time::default())
                .register::<Ball>()
                .register::<Paddle>()
                .with(PaddleSystem, "paddle_system", &["input_system"])
                .with(MoveBallsSystem, "ball_system", &[])
                .with(
                    BounceSystem,
                    "collision_system",
                    &["paddle_system", "ball_system"],
                )
                .with(
                    WinnerSystem,
                    "winner_system",
                    &["paddle_system", "ball_system"],
                ),
        )
    }
}
