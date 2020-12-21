//! ECS transform bundle

use amethyst_error::Error;

use crate::{ecs::*, transform::*};

/// Transform bundle
#[derive(Default)]
#[allow(missing_debug_implementations)]
pub struct TransformBundle;

impl SystemBundle for TransformBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder
            .add_system(Box::new(MissingPreviousParentSystem))
            .add_system(Box::new(ParentUpdateSystem))
            .add_system(Box::new(TransformSystem));

        Ok(())
    }
}
