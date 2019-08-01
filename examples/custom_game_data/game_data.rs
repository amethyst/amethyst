use std::marker::PhantomData;

use amethyst::{
    core::{ArcThreadPool, SystemBundle, SystemDesc},
    ecs::prelude::{Dispatcher, DispatcherBuilder, System, World, WorldExt},
    error::Error,
    DataDispose, DataInit,
};

pub struct CustomGameData<'a, 'b> {
    pub base: Option<Dispatcher<'a, 'b>>,
    pub running: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> CustomGameData<'a, 'b> {
    /// Update game data
    pub fn update(&mut self, world: &World, running: bool) {
        if running {
            if let Some(running) = &mut self.running {
                running.dispatch(&world);
            }
        }
        if let Some(base) = &mut self.base {
            base.dispatch(&world);
        }
    }

    /// Dispose game data, dropping the dispatcher
    pub fn dispose(&mut self, world: &mut World) {
        if let Some(base) = self.base.take() {
            base.dispose(world);
        }
        if let Some(running) = self.running.take() {
            running.dispose(world);
        }
    }
}

impl DataDispose for CustomGameData<'_, '_> {
    fn dispose(&mut self, world: &mut World) {
        self.dispose(world);
    }
}

pub struct CustomGameDataBuilder<'a, 'b> {
    base_dispatcher_operations: Vec<Box<dyn DispatcherOperation<'a, 'b>>>,
    running_dispatcher_operations: Vec<Box<dyn DispatcherOperation<'a, 'b>>>,
}

impl<'a, 'b> Default for CustomGameDataBuilder<'a, 'b> {
    fn default() -> Self {
        CustomGameDataBuilder::new()
    }
}

impl<'a, 'b> CustomGameDataBuilder<'a, 'b> {
    pub fn new() -> Self {
        CustomGameDataBuilder {
            base_dispatcher_operations: vec![],
            running_dispatcher_operations: vec![],
        }
    }

    pub fn with_base<SD, S>(
        mut self,
        system_desc: SD,
        name: &'static str,
        dependencies: &'static [&'static str],
    ) -> Self
    where
        SD: SystemDesc<'a, 'b, S> + 'static,
        S: for<'c> System<'c> + 'static + Send,
    {
        let dispatcher_operation = Box::new(AddSystem {
            system_desc,
            name,
            dependencies,
            marker: PhantomData::<S>,
        }) as Box<dyn DispatcherOperation<'a, 'b> + 'static>;
        self.base_dispatcher_operations.push(dispatcher_operation);
        self
    }

    pub fn with_base_bundle<B>(mut self, bundle: B) -> Self
    where
        B: SystemBundle<'a, 'b> + 'static,
    {
        self.base_dispatcher_operations
            .push(Box::new(AddBundle { bundle }));
        self
    }

    pub fn with_running<SD, S>(
        mut self,
        system_desc: SD,
        name: &'static str,
        dependencies: &'static [&'static str],
    ) -> Self
    where
        SD: SystemDesc<'a, 'b, S> + 'static,
        S: for<'c> System<'c> + 'static + Send,
    {
        let dispatcher_operation = Box::new(AddSystem {
            system_desc,
            name,
            dependencies,
            marker: PhantomData::<S>,
        }) as Box<dyn DispatcherOperation<'a, 'b> + 'static>;
        self.running_dispatcher_operations
            .push(dispatcher_operation);
        self
    }
}

impl<'a, 'b> DataInit<CustomGameData<'a, 'b>> for CustomGameDataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> CustomGameData<'a, 'b> {
        let base = build_dispatcher(world, self.base_dispatcher_operations);
        let running = build_dispatcher(world, self.running_dispatcher_operations);

        CustomGameData {
            base: Some(base),
            running: Some(running),
        }
    }
}

fn build_dispatcher<'a, 'b>(
    world: &mut World,
    dispatcher_operations: Vec<Box<dyn DispatcherOperation<'a, 'b>>>,
) -> Dispatcher<'a, 'b> {
    let mut dispatcher_builder = DispatcherBuilder::new();

    #[cfg(not(no_threading))]
    {
        let pool = world.read_resource::<ArcThreadPool>().clone();
        dispatcher_builder = dispatcher_builder.with_pool((*pool).clone());
    }

    dispatcher_operations
        .into_iter()
        .try_for_each(|dispatcher_operation| {
            dispatcher_operation.exec(world, &mut dispatcher_builder)
        })
        .unwrap_or_else(|e| panic!("Failed to set up dispatcher: {}", e));

    let mut dispatcher = dispatcher_builder.build();
    dispatcher.setup(world);
    dispatcher
}

/// Trait to capture deferred dispatcher builder operations.
trait DispatcherOperation<'a, 'b> {
    /// Executes the dispatcher builder instruction.
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error>;
}

struct AddSystem<SD, S> {
    system_desc: SD,
    name: &'static str,
    dependencies: &'static [&'static str],
    marker: PhantomData<S>,
}

impl<'a, 'b, SD, S> DispatcherOperation<'a, 'b> for AddSystem<SD, S>
where
    SD: SystemDesc<'a, 'b, S>,
    S: for<'s> System<'s> + Send + 'a,
{
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let system = self.system_desc.build(world);
        dispatcher_builder.add(system, self.name, self.dependencies);
        Ok(())
    }
}

struct AddBundle<B> {
    bundle: B,
}

impl<'a, 'b, B> DispatcherOperation<'a, 'b> for AddBundle<B>
where
    B: SystemBundle<'a, 'b>,
{
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        self.bundle.build(world, dispatcher_builder)?;
        Ok(())
    }
}
