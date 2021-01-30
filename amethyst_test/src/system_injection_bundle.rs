use amethyst::{core::bundle::SystemBundle, ecs::*, error::Error};
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
    system_name: String,
    /// Names of the system dependencies.
    system_dependencies: Vec<String>,
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
        let system_dependencies = self
            .system_dependencies
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>();
        builder.add(self.system, &self.system_name, &system_dependencies);
        Ok(())
    }
}
