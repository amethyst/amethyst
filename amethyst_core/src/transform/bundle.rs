//! ECS transform bundle

use amethyst_error::Error;

use crate::{ecs::*, transform::*};

/// Transform bundle
pub struct TransformBundle;

impl SystemBundle for TransformBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder
            .add_system(missing_previous_parent_system::build())
            .add_system(parent_update_system::build())
            .add_system(transform_system::build());

        Ok(())
    }
}
