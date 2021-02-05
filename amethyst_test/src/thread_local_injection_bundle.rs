use std::marker::PhantomData;

use amethyst::{
    core::{bundle::SystemBundle, RunNowDesc},
    ecs::*,
    error::Error,
};
use derive_new::new;

/// Adds a specified thread local `System` to the dispatcher.
#[derive(Debug, new)]
pub struct ThreadLocalInjectionBundle<'a, 'b, RNDesc, RN>
where
    RNDesc: RunNowDesc<'a, 'b, RN>,
    RN: for<'s> RunNow<'s>,
{
    /// Function to instantiate `System` to add to the dispatcher.
    run_now_desc: RNDesc,
    /// Marker.
    #[new(default)]
    system_marker: PhantomData<(&'a RNDesc, &'b RN)>,
}

impl<'a, 'b, RNDesc, RN> SystemBundle<'a, 'b> for ThreadLocalInjectionBundle<'a, 'b, RNDesc, RN>
where
    RNDesc: RunNowDesc<'a, 'b, RN>,
    RN: for<'s> RunNow<'s> + 'b,
{
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let system = self.run_now_desc.build(world);
        builder.add_thread_local(system);
        Ok(())
    }
}
