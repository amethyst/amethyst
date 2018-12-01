use amethyst::{
    core::bundle::{Result, SystemBundle},
    GameDataBuilder,
};
use crate::systems::{BounceSystem, MoveBallsSystem, PaddleSystem, WinnerSystem};

/// A bundle is a convenient way to initialise related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
#[derive(Default)]
pub struct PongBundle;

impl<'a, 'b, 'c> SystemBundle<'a, 'b, 'c, GameDataBuilder<'a, 'b, 'c>> for PongBundle {
    fn build(self, builder: &mut GameDataBuilder<'a, 'b, 'c>) -> Result<()> {
        builder.add(PaddleSystem, "paddle_system", &["input_system"], &[]);
        builder.add(MoveBallsSystem, "ball_system", &[], &[]);
        builder.add(
            BounceSystem,
            "collision_system",
            &["paddle_system", "ball_system"],
            &[],
        );
        builder.add(
            WinnerSystem,
            "winner_system",
            &["paddle_system", "ball_system"],
            &[],
        );
        Ok(())
    }
}
