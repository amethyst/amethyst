use std::marker::PhantomData;

use amethyst::{
    core::{bundle::SystemBundle, SystemDesc},
    ecs::prelude::*,
    error::Error,
};
use derive_new::new;

/// Adds a specified thread local `System` to the dispatcher.
#[derive(Debug, new)]
pub struct ThreadLocalInjectionBundle<'a, 'b, SD, S>
where
    SD: SystemDesc<'a, 'b, S>,
    S: for<'s> System<'s>,
    // S: for<'s> RunNow<'s>,
{
    /// Function to instantiate `System` to add to the dispatcher.
    system_desc: SD,
    /// Marker for `'a` lifetime.
    #[new(default)]
    system_marker: PhantomData<(&'a SD, &'b S)>,
}

impl<'a, 'b, SD, S> SystemBundle<'a, 'b> for ThreadLocalInjectionBundle<'a, 'b, SD, S>
where
    SD: SystemDesc<'a, 'b, S>,
    S: for<'s> System<'s>,
    // S: for<'s> RunNow<'s> + 'b,
{
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let system = self.system_desc.build(world);
        builder.add_thread_local(system);
        Ok(())
    }
}
