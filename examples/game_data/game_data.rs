use std::sync::Arc;

use amethyst::ecs::prelude::{Dispatcher, DispatcherBuilder, System, World};
use amethyst::core::SystemBundle;
use amethyst::{DataInit, Error, Result};
use rayon::ThreadPool;

pub struct GameData<'a, 'b> {
    pub base: Dispatcher<'a, 'b>,
    pub running: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameData<'a, 'b> {
    /// Update game data
    pub fn update(&mut self, world: &World, running: bool) {
        if running {
            self.running.dispatch(&world.res);
        }
        self.base.dispatch(&world.res);
    }
}

pub struct GameDataBuilder<'a, 'b> {
    pub base: DispatcherBuilder<'a, 'b>,
    pub running: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> Default for GameDataBuilder<'a, 'b> {
    fn default() -> Self {
        GameDataBuilder::new()
    }
}

impl<'a, 'b> GameDataBuilder<'a, 'b> {
    pub fn new() -> Self {
        GameDataBuilder {
            base: DispatcherBuilder::new(),
            running: DispatcherBuilder::new(),
        }
    }

    pub fn with_base_bundle<B>(mut self, bundle: B) -> Result<Self>
    where
        B: SystemBundle<'a, 'b>,
    {
        bundle
            .build(&mut self.base)
            .map_err(|err| Error::Core(err))?;
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

impl<'a, 'b> DataInit<GameData<'a, 'b>> for GameDataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> GameData<'a, 'b> {
        #[cfg(not(no_threading))]
        let pool = world.read_resource::<Arc<ThreadPool>>().clone();

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

        GameData { base, running }
    }
}
