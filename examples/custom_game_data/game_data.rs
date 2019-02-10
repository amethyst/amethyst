use amethyst::{
    core::{ArcThreadPool, SystemBundle},
    ecs::prelude::{Dispatcher, DispatcherBuilder, System, World},
    error::Error,
    DataInit,
};

pub struct CustomGameData<'a, 'b> {
    pub base: Dispatcher<'a, 'b>,
    pub running: Dispatcher<'a, 'b>,
}

impl<'a, 'b> CustomGameData<'a, 'b> {
    /// Update game data
    pub fn update(&mut self, world: &World, running: bool) {
        if running {
            self.running.dispatch(&world.res);
        }
        self.base.dispatch(&world.res);
    }
}

pub struct CustomGameDataBuilder<'a, 'b> {
    pub base: DispatcherBuilder<'a, 'b>,
    pub running: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> Default for CustomGameDataBuilder<'a, 'b> {
    fn default() -> Self {
        CustomGameDataBuilder::new()
    }
}

impl<'a, 'b> CustomGameDataBuilder<'a, 'b> {
    pub fn new() -> Self {
        CustomGameDataBuilder {
            base: DispatcherBuilder::new(),
            running: DispatcherBuilder::new(),
        }
    }

    pub fn with_base<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Self
    where
        for<'c> S: System<'c> + Send + 'a,
    {
        self.base.add(system, name, dependencies);
        self
    }

    pub fn with_base_bundle<B>(mut self, bundle: B) -> Result<Self, Error>
    where
        B: SystemBundle<'a, 'b>,
    {
        bundle.build(&mut self.base)?;
        Ok(self)
    }

    pub fn with_running<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Self
    where
        for<'c> S: System<'c> + Send + 'a,
    {
        self.running.add(system, name, dependencies);
        self
    }
}

impl<'a, 'b> DataInit<CustomGameData<'a, 'b>> for CustomGameDataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> CustomGameData<'a, 'b> {
        #[cfg(not(no_threading))]
        let pool = world.read_resource::<ArcThreadPool>().clone();

        #[cfg(not(no_threading))]
        let mut base = self.base.with_pool(pool.clone()).build();
        #[cfg(no_threading)]
        let mut base = self.base.build();
        base.setup(&mut world.res);

        #[cfg(not(no_threading))]
        let mut running = self.running.with_pool(pool.clone()).build();
        #[cfg(no_threading)]
        let mut running = self.running.build();
        running.setup(&mut world.res);

        CustomGameData { base, running }
    }
}
