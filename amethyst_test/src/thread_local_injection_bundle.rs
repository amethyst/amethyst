use std::marker::PhantomData;

use amethyst::{core::bundle::SystemBundle, ecs::prelude::*, error::Error};
use derive_new::new;

/// Adds a specified thread local `System` to the dispatcher.
#[derive(Debug, new)]
pub struct ThreadLocalInjectionBundle<Sys>
where
    Sys: for<'s> RunNow<'s>,
{
    /// `System` to add to the dispatcher.
    system: Sys,
    /// Marker for `'a` lifetime.
    #[new(default)]
    system_marker: PhantomData<Sys>,
}

impl<'a, 'b, Sys> SystemBundle<'a, 'b> for ThreadLocalInjectionBundle<Sys>
where
    Sys: for<'s> RunNow<'s> + 'b,
{
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add_thread_local(self.system);
        Ok(())
    }
}
