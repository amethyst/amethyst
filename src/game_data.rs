use std::marker::PhantomData;

use crate::{
    core::{
        dispatcher::{Dispatcher, DispatcherBuilder, IntoRelativeStage, SystemBundle, ThreadLocal},
        ecs::prelude::*,
        ArcThreadPool,
    },
    error::Error,
};

/// Initialise trait for game data
pub trait DataInit<T> {
    /// Build game data
    fn build(self, world: &mut World, resources: &mut Resources) -> T;
}

/// Allow disposing game data with access to world.
pub trait DataDispose {
    /// Perform disposal
    fn dispose(&mut self, world: &mut World, resources: &mut Resources);
}

/// Default game data.
///
/// The lifetimes are for the systems inside and can be `'static` unless a system has a borrowed
/// field.
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

    /// Update game data
    pub fn update(&mut self, world: &mut World, resources: &mut Resources) {
        if let Some(dispatcher) = &mut self.dispatcher {
            dispatcher.dispatch(world, resources);
        }
    }

    /// Dispose game data, dropping the dispatcher
    pub fn dispose(&mut self, world: &mut World, resources: &mut Resources) {
        if let Some(dispatcher) = self.dispatcher.take() {
            dispatcher.dispose(world, resources);
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

/// Builder for default game data
#[allow(missing_debug_implementations)]
pub struct GameDataBuilder<'a> {
    disp_builder: DispatcherBuilder<'a>,
}

impl<'a> Default for GameDataBuilder<'a> {
    fn default() -> Self {
        GameDataBuilder::new()
    }
}

impl<'a> GameDataBuilder<'a> {
    /// Create new builder
    pub fn new() -> Self {
        GameDataBuilder {
            disp_builder: DispatcherBuilder::default(),
        }
    }

    pub fn with_thread_local<T: FnOnce(&mut World, &mut Resources) -> Box<dyn ThreadLocal> + 'a>(
        mut self,
        desc: T,
    ) -> Self {
        self.disp_builder.add_thread_local(desc);

        self
    }

    pub fn with_system<
        S: IntoRelativeStage,
        T: FnOnce(&mut World, &mut Resources) -> Box<dyn Schedulable> + 'a,
    >(
        mut self,
        stage: S,
        desc: T,
    ) -> Self {
        self.disp_builder.add_system(stage, desc);

        self
    }

    pub fn with_bundle<T: SystemBundle + 'a>(mut self, bundle: T) -> Self {
        self.disp_builder.add_bundle(bundle);

        self
    }
}

impl<'a> DataInit<GameData> for GameDataBuilder<'a> {
    // /// Create a basic renderer with a single given `Pass`, and optional support for the `DrawUi` pass.
    // ///
    // /// Will set the clear color to black.
    // ///
    // /// ### Parameters:
    // ///
    // /// - `path`: Path to the `DisplayConfig` configuration file
    // /// - `pass`: The single pass in the render graph
    // /// - `with_ui`: If set to true, will add the UI render pass
    // pub fn with_basic_renderer<A, P>(self, path: A, pass: P, with_ui: bool) -> Result<Self, Error>
    // where
    //     A: AsRef<Path>,
    //     P: Pass + 'b,
    // {
    //     use crate::{
    //         config::Config,
    //         renderer::{DisplayConfig, Pipeline, RenderBundle, Stage},
    //         ui::DrawUi,
    //     };
    //     let config = DisplayConfig::load(path);
    //     if with_ui {
    //         let pipe = Pipeline::build().with_stage(
    //             Stage::with_backbuffer()
    //                 .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
    //                 .with_pass(pass)
    //                 .with_pass(DrawUi::new()),
    //         );
    //         self.with_bundle(RenderBundle::new(pipe, Some(config)))
    //     } else {
    //         let pipe = Pipeline::build().with_stage(
    //             Stage::with_backbuffer()
    //                 .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
    //                 .with_pass(pass),
    //         );
    //         self.with_bundle(RenderBundle::new(pipe, Some(config)))
    //     }
    // }
    fn build(self, world: &mut World, resources: &mut Resources) -> GameData {
        #[cfg(not(no_threading))]
        let pool = (*resources.get::<ArcThreadPool>().unwrap()).clone();

        let mut dispatcher_builder = self.disp_builder;

        #[cfg(not(no_threading))]
        let mut dispatcher = dispatcher_builder
            .with_pool(Some(pool))
            .build(world, resources);
        #[cfg(no_threading)]
        let mut dispatcher = dispatcher_builder.build(world, resources);

        GameData::new(dispatcher)
    }
}

impl DataInit<()> for () {
    fn build(self, _: &mut World, _: &mut Resources) {}
}
