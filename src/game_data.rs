use std::marker::PhantomData;
use std::path::Path;

use crate::{
    core::{
        ecs::prelude::{Dispatcher, DispatcherBuilder, System, World},
        math::RealField,
        ArcThreadPool, SystemBundle,
    },
    error::Error,
    renderer::pipe::pass::Pass,
};

/// Initialise trait for game data
pub trait DataInit<T> {
    /// Build game data
    fn build(self, world: &mut World) -> T;
}

/// Default game data.
///
/// The lifetimes are for the systems inside and can be `'static` unless a system has a borrowed
/// field.
pub struct GameData<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameData<'a, 'b> {
    /// Create new game data
    pub fn new(dispatcher: Dispatcher<'a, 'b>) -> Self {
        GameData { dispatcher }
    }

    /// Update game data
    pub fn update(&mut self, world: &World) {
        self.dispatcher.dispatch(&world.res);
    }
}

/// Builder for default game data
pub struct GameDataBuilder<'a, 'b, N: RealField = f32> {
    disp_builder: DispatcherBuilder<'a, 'b>,
    _marker: PhantomData<N>,
}

impl<'a, 'b, N: RealField + Default> Default for GameDataBuilder<'a, 'b, N> {
    fn default() -> Self {
        GameDataBuilder::new()
    }
}

impl<'a, 'b, N: RealField + Default> GameDataBuilder<'a, 'b, N> {
    /// Create new builder
    pub fn new() -> Self {
        GameDataBuilder {
            disp_builder: DispatcherBuilder::new(),
            _marker: PhantomData,
        }
    }

    /// Inserts a barrier which assures that all systems added before the
    /// barrier are executed before the ones after this barrier.
    ///
    /// Does nothing if there were no systems added since the last call to
    /// `with_barrier()`. Thread-local systems are not affected by barriers;
    /// they're always executed at the end.
    ///
    /// # Returns
    ///
    /// This function returns GameDataBuilder after it has modified it.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::prelude::System;
    ///
    /// struct NopSystem;
    /// impl<'a> System<'a> for NopSystem {
    ///     type SystemData = ();
    ///     fn run(&mut self, (): Self::SystemData) {}
    /// }
    ///
    /// // Three systems are added in this example. The "tabby cat" & "tom cat"
    /// // systems will both run in parallel. Only after both cat systems have
    /// // run is the "doggo" system permitted to run them.
    /// GameDataBuilder::<f32>::default()
    ///     .with(NopSystem, "tabby cat", &[])
    ///     .with(NopSystem, "tom cat", &[])
    ///     .with_barrier()
    ///     .with(NopSystem, "doggo", &[]);
    /// ~~~
    pub fn with_barrier(mut self) -> Self {
        self.disp_builder.add_barrier();
        self
    }

    /// Adds a given system.
    ///
    /// __Note:__ all dependencies must be added before you add the system.
    ///
    /// # Parameters
    ///
    /// - `system`: The system that is to be added to the game loop.
    /// - `name`: A unique string to identify the system by. This is used for
    ///         dependency tracking. This name may be empty `""` string in which
    ///         case it cannot be referenced as a dependency.
    /// - `dependencies`: A list of named system that _must_ have completed running
    ///                 before this system is permitted to run.
    ///                 This may be an empty list if there is no dependencies.
    ///
    /// # Returns
    ///
    /// This function returns GameDataBuilder after it has modified it.
    ///
    /// # Type Parameters
    ///
    /// - `S`: A type that implements the `System` trait.
    ///
    /// # Panics
    ///
    /// If two system are added that share an identical name, this function will panic.
    /// Empty names are permitted, and this function will not panic if more then two are added.
    ///
    /// If a dependency is referenced (by name), but has not previously been added this
    /// function will panic.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::prelude::System;
    ///
    /// struct NopSystem;
    /// impl<'a> System<'a> for NopSystem {
    ///     type SystemData = ();
    ///     fn run(&mut self, _: Self::SystemData) {}
    /// }
    ///
    /// GameDataBuilder::<f32>::default()
    ///     // This will add the "foo" system to the game loop, in this case
    ///     // the "foo" system will not depend on any systems.
    ///     .with(NopSystem, "foo", &[])
    ///     // The "bar" system will only run after the "foo" system has completed
    ///     .with(NopSystem, "bar", &["foo"])
    ///     // It is legal to register a system with an empty name
    ///     .with(NopSystem, "", &[]);
    /// ~~~
    pub fn with<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Self
    where
        for<'c> S: System<'c> + Send + 'a,
    {
        self.disp_builder.add(system, name, dependencies);
        self
    }

    /// Add a given thread-local system.
    ///
    /// A thread-local system is one that _must_ run on the main thread of the
    /// game. A thread-local system would be necessary typically to work
    /// around vendor APIs that have thread dependent designs; an example
    /// being OpenGL which uses a thread-local state machine to function.
    ///
    /// All thread-local systems are executed sequentially after all
    /// non-thread-local systems.
    ///
    /// # Parameters
    ///
    /// - `system`: The system that is to be added to the game loop.
    ///
    /// # Returns
    ///
    /// This function returns GameDataBuilder after it has modified it.
    ///
    /// # Type Parameters
    ///
    /// - `S`: A type that implements the `System` trait.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::prelude::System;
    ///
    /// struct NopSystem;
    /// impl<'a> System<'a> for NopSystem {
    ///     type SystemData = ();
    ///     fn run(&mut self, _: Self::SystemData) {}
    /// }
    ///
    /// GameDataBuilder::<f32>::default()
    ///     // the Nop system is registered here
    ///     .with_thread_local(NopSystem);
    /// ~~~
    pub fn with_thread_local<S>(mut self, system: S) -> Self
    where
        for<'c> S: System<'c> + 'b,
    {
        self.disp_builder.add_thread_local(system);
        self
    }

    /// Add a given ECS bundle to the game loop.
    ///
    /// A bundle is a container for registering a bunch of ECS systems at once.
    ///
    /// # Parameters
    ///
    /// - `bundle`: The bundle to add
    ///
    /// # Returns
    ///
    /// This function returns GameDataBuilder after it has modified it, this is
    /// wrapped in a `Result`.
    ///
    /// # Errors
    ///
    /// This function creates systems, which use any number of dependent crates or APIs, which
    /// could result in any number of errors.
    /// See each individual bundle for a description of the errors it could produce.
    ///
    pub fn with_bundle<B>(mut self, bundle: B) -> Result<Self, Error>
    where
        B: SystemBundle<'a, 'b>,
    {
        bundle.build(&mut self.disp_builder)?;
        Ok(self)
    }

    /// Create a basic renderer with a single given `Pass`, and optional support for the `DrawUi` pass.
    ///
    /// Will set the clear color to black.
    ///
    /// ### Parameters:
    ///
    /// - `path`: Path to the `DisplayConfig` configuration file
    /// - `pass`: The single pass in the render graph
    /// - `with_ui`: If set to true, will add the UI render pass
    pub fn with_basic_renderer<A, P>(self, path: A, pass: P, with_ui: bool) -> Result<Self, Error>
    where
        A: AsRef<Path>,
        P: Pass + 'b,
    {
        use crate::{
            config::Config,
            renderer::{DisplayConfig, Pipeline, RenderBundle, Stage},
            ui::DrawUi,
        };
        let config = DisplayConfig::load(path);
        if with_ui {
            let pipe = Pipeline::build().with_stage(
                Stage::with_backbuffer()
                    .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                    .with_pass(pass)
                    .with_pass(DrawUi::new()),
            );
            self.with_bundle(RenderBundle::<'_, _, _, N>::new(pipe, Some(config)))
        } else {
            let pipe = Pipeline::build().with_stage(
                Stage::with_backbuffer()
                    .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                    .with_pass(pass),
            );
            self.with_bundle(RenderBundle::<'_, _, _, N>::new(pipe, Some(config)))
        }
    }
}

impl<'a, 'b> DataInit<GameData<'a, 'b>> for GameDataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> GameData<'a, 'b> {
        #[cfg(not(no_threading))]
        let pool = world.read_resource::<ArcThreadPool>().clone();

        #[cfg(not(no_threading))]
        let mut dispatcher = self.disp_builder.with_pool(pool).build();
        #[cfg(no_threading)]
        let mut dispatcher = self.disp_builder.build();
        dispatcher.setup(&mut world.res);
        GameData::new(dispatcher)
    }
}

impl DataInit<()> for () {
    fn build(self, _: &mut World) {}
}
