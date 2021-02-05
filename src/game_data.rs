use crate::{core::ecs::*, error::Error};

/// initialize trait for game data
pub trait DataInit<T> {
    /// Build game data
    fn build(self, world: &mut World, resources: &mut Resources) -> Result<T, Error>;
}

/// Allow disposing game data with access to world.
pub trait DataDispose {
    /// Perform disposal
    fn dispose(&mut self, world: &mut World, resources: &mut Resources);
}

/// Default game data.
#[allow(missing_debug_implementations)]
pub struct GameData {
    dispatcher: Option<Dispatcher>,
}

impl GameData {
    /// Create new game data
    pub fn new(dispatcher: Dispatcher) -> Self {
        GameData {
            dispatcher: Some(dispatcher),
        }
    }

    /// Update game data by executing internal [Dispatcher]
    pub fn update(&mut self, world: &mut World, resources: &mut Resources) {
        if let Some(dispatcher) = &mut self.dispatcher {
            dispatcher.execute(world, resources);
        }
    }

    /// Dispose game data, dropping the dispatcher
    pub fn dispose(&mut self, world: &mut World, resources: &mut Resources) {
        if let Some(dispatcher) = self.dispatcher.take() {
            dispatcher.unload(world, resources).unwrap();
        }
    }
}

impl DataDispose for () {
    fn dispose(&mut self, _world: &mut World, _resources: &mut Resources) {}
}

impl DataDispose for GameData {
    fn dispose(&mut self, world: &mut World, resources: &mut Resources) {
        self.dispose(world, resources);
    }
}

impl DataInit<GameData> for DispatcherBuilder {
    fn build(mut self, world: &mut World, resources: &mut Resources) -> Result<GameData, Error> {
        let dispatcher = DispatcherBuilder::build(&mut self, world, resources)?;
        Ok(GameData::new(dispatcher))
    }
}

impl DataInit<()> for () {
    fn build(self, _: &mut World, _: &mut Resources) -> Result<(), Error> {
        Ok(())
    }
}
