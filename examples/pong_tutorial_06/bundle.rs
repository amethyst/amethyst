use crate::systems::{BounceSystem, MoveBallsSystem, PaddleSystem, WinnerSystem};
use amethyst::{
    core::bundle::SystemBundle,
    ecs::prelude::{DispatcherBuilder, World},
    error::Error,
};

/// A bundle is a convenient way to initialise related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
pub struct PongBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PongBundle {
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(PaddleSystem, "paddle_system", &["input_system"]);
        builder.add(MoveBallsSystem, "ball_system", &[]);
        builder.add(
            BounceSystem,
            "collision_system",
            &["paddle_system", "ball_system"],
        );
        builder.add(
            WinnerSystem,
            "winner_system",
            &["paddle_system", "ball_system"],
        );
        Ok(())
    }
}
