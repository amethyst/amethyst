use std::marker::PhantomData;

use amethyst::{core::bundle::SystemBundle, ecs::prelude::*, error::Error};

use derive_new::new;

/// Adds a specified `System` to the dispatcher.
#[derive(Debug, new)]
pub(crate) struct SystemInjectionBundle<'a, SysFn, Sys>
where
    SysFn: FnOnce(&mut World) -> Sys,
    Sys: for<'s> System<'s> + Send,
{
    /// Function to instantiate `System` to add to the dispatcher.
    system_fn: SysFn,
    /// Name to register the system with.
    system_name: String,
    /// Names of the system dependencies.
    system_dependencies: Vec<String>,
    /// Marker for `'a` lifetime.
    #[new(default)]
    system_marker: PhantomData<(SysFn, &'a Sys)>,
}

impl<'a, 'b, SysFn, Sys> SystemBundle<'a, 'b> for SystemInjectionBundle<'a, SysFn, Sys>
where
    SysFn: FnOnce(&mut World) -> Sys,
    Sys: for<'s> System<'s> + Send + 'a,
{
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            (self.system_fn)(world),
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
