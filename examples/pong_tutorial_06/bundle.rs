use amethyst::{
    core::bundle::SystemBundle,
    ecs::{DispatcherBuilder, World},
    error::Error,
};

use crate::systems::{BounceSystem, MoveBallsSystem, PaddleSystem, WinnerSystem};

/// A bundle is a convenient way to initialise related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
pub struct PongBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PongBundle {
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder
            .add_system(Box::new(systems::paddle::build()))
            .add_system(Box::new(systems::move_balls::build()))
            .flush()
            .add_system(Box::new(systems::bounce::build()))
            .add_system(Box::new(systems::winner::build()));

        Ok(())
    }
}
