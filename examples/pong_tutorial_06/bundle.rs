use amethyst::{
    core::ecs::SystemBundle,
    ecs::{DispatcherBuilder, Resources, World},
    error::Error,
};

use crate::systems::{
    bounce::BounceSystem, move_balls::BallSystem, paddle::PaddleSystem, winner::WinnerSystem,
};

/// A bundle is a convenient way to initialize related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
pub struct PongBundle;

impl SystemBundle for PongBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder
            .add_system(PaddleSystem)
            .add_system(BallSystem)
            .flush()
            .add_system(BounceSystem)
            .add_system(WinnerSystem);
        Ok(())
    }
}
