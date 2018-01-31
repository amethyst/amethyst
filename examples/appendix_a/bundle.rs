use {Ball, Paddle, ScoreBoard};
use amethyst::config::Config;
use amethyst::core::bundle::{ECSBundle, Result};
use amethyst::core::timing::Time;
use amethyst::ecs::{DispatcherBuilder, World};
use config::PongConfig;
use std::path::Path;
use systems::{BounceSystem, MoveBallsSystem, PaddleSystem, WinnerSystem};

/// A bundle is a convenient way to initialise related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
pub struct PongBundle {
    config: PongConfig,
}

impl PongBundle {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        PongBundle {
            config: PongConfig::load(path),
        }
    }
}

impl<'a, 'b> ECSBundle<'a, 'b> for PongBundle {
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(self.config.arena);
        world.add_resource(self.config.ball);
        world.add_resource(self.config.paddles);
        world.add_resource(ScoreBoard::new());
        world.add_resource(Time::default());
        world.register::<Ball>();
        world.register::<Paddle>();

        Ok(builder
            .add(PaddleSystem, "paddle_system", &["input_system"])
            .add(MoveBallsSystem, "ball_system", &[])
            .add(
                BounceSystem,
                "collision_system",
                &["paddle_system", "ball_system"],
            )
            .add(
                WinnerSystem,
                "winner_system",
                &["paddle_system", "ball_system"],
            ))
    }
}
