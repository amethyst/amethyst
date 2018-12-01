use std::marker::PhantomData;

use amethyst::core::SimpleDispatcherBuilder;
use amethyst::core::bundle::{Result, SystemBundle};
use amethyst::ecs::prelude::*;

/// Adds a specified `System` to the dispatcher.
#[derive(Debug, new)]
pub(crate) struct SystemInjectionBundle<'a, 'b, Sys>
where
    Sys: for<'s> System<'s> + Send + 'a,
{
    /// `System` to add to the dispatcher.
    system: Sys,
    /// Name to register the system with.
    system_name: &'b str,
    /// Names of the system dependencies.
    system_dependencies: Vec<&'b str>,
    /// Marker for `'a` lifetime.
    #[new(default)]
    system_marker: PhantomData<&'a Sys>,
}

impl<'a, 'b, 'c, D, Sys> SystemBundle<'a, 'b, 'c, D> for SystemInjectionBundle<'a, 'c, Sys>
where
    Sys: for<'s> System<'s> + Send + 'a + 'c,
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    fn build(self, builder: &mut D) -> Result<()> {
        builder.add(
            self.system,
            &self.system_name,
            &self.system_dependencies,
        );
        Ok(())
    }
}
