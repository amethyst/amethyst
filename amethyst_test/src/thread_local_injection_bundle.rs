use std::marker::PhantomData;

use amethyst::{core::bundle::SystemBundle, ecs::prelude::*, error::Error};
use derive_new::new;

/// Adds a specified thread local `System` to the dispatcher.
#[derive(Debug, new)]
pub struct ThreadLocalInjectionBundle<SysFn, Sys>
where
    SysFn: FnOnce(&mut World) -> Sys,
    Sys: for<'s> RunNow<'s>,
{
    /// Function to instantiate `System` to add to the dispatcher.
    system_fn: SysFn,
    /// Marker for `'a` lifetime.
    #[new(default)]
    system_marker: PhantomData<(SysFn, Sys)>,
}

impl<'a, 'b, SysFn, Sys> SystemBundle<'a, 'b> for ThreadLocalInjectionBundle<SysFn, Sys>
where
    SysFn: FnOnce(&mut World) -> Sys,
    Sys: for<'s> RunNow<'s> + 'b,
{
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let system = (self.system_fn)(world);
        builder.add_thread_local(system);
        Ok(())
    }
}
