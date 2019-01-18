use std::marker::PhantomData;

use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::*,
};

use derive_new::new;

/// Adds a specified `System` to the dispatcher.
#[derive(Debug, new)]
pub(crate) struct SystemInjectionBundle<'a, Sys>
where
    Sys: for<'s> System<'s> + Send,
{
    /// `System` to add to the dispatcher.
    system: Sys,
    /// Name to register the system with.
    system_name: String,
    /// Names of the system dependencies.
    system_dependencies: Vec<String>,
    /// Marker for `'a` lifetime.
    #[new(default)]
    system_marker: PhantomData<&'a Sys>,
}

impl<'a, 'b, Sys> SystemBundle<'a, 'b> for SystemInjectionBundle<'a, Sys>
where
    Sys: for<'s> System<'s> + Send + 'a,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            self.system,
            &self.system_name,
            &self
                .system_dependencies
                .iter()
                .map(|dep| dep.as_str())
                .collect::<Vec<&str>>(),
        );
        Ok(())
    }
}
