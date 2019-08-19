use amethyst::{core::bundle::SystemBundle, ecs::prelude::*, error::Error};

use derive_new::new;

/// Adds a specified `System` to the dispatcher.
#[derive(Debug, new)]
pub(crate) struct SystemInjectionBundle<S>
where
    S: for<'s> System<'s> + Send,
{
    /// Function to instantiate `System` to add to the dispatcher.
    system: S,
    /// Name to register the system with.
    system_name: &'static str,
    /// Names of the system dependencies.
    system_dependencies: &'static [&'static str],
}

impl<'a, 'b, S> SystemBundle<'a, 'b> for SystemInjectionBundle<S>
where
    S: for<'s> System<'s> + Send + 'a,
{
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(self.system, self.system_name, self.system_dependencies);
        Ok(())
    }
}
