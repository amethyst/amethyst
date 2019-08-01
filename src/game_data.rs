use std::marker::PhantomData;

use derivative::Derivative;

use crate::{
    core::{
        ecs::prelude::{Dispatcher, DispatcherBuilder, System, World, WorldExt},
        ArcThreadPool, SystemBundle, SystemDesc,
    },
    error::Error,
};

/// Initialise trait for game data
pub trait DataInit<T> {
    /// Build game data
    fn build(self, world: &mut World) -> T;
}

/// Allow disposing game data with access to world.
pub trait DataDispose {
    /// Perform disposal
    fn dispose(&mut self, world: &mut World);
}

/// Default game data.
///
/// The lifetimes are for the systems inside and can be `'static` unless a system has a borrowed
/// field.
#[allow(missing_debug_implementations)]
pub struct GameData<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> GameData<'a, 'b> {
    /// Create new game data
    pub fn new(dispatcher: Dispatcher<'a, 'b>) -> Self {
        GameData {
            dispatcher: Some(dispatcher),
        }
    }

    /// Update game data
    pub fn update(&mut self, world: &World) {
        if let Some(dispatcher) = &mut self.dispatcher {
            dispatcher.dispatch(&world);
        }
    }

    /// Dispose game data, dropping the dispatcher
    pub fn dispose(&mut self, mut world: &mut World) {
        if let Some(dispatcher) = self.dispatcher.take() {
            dispatcher.dispose(&mut world);
        }
    }
}

impl DataDispose for () {
    fn dispose(&mut self, _world: &mut World) {}
}

impl DataDispose for GameData<'_, '_> {
    fn dispose(&mut self, world: &mut World) {
        self.dispose(world);
    }
}

/// Builder for default game data
#[allow(missing_debug_implementations)]
pub struct GameDataBuilder<'a, 'b> {
    dispatcher_operations: Vec<Box<dyn DispatcherOperation<'a, 'b>>>,
    disp_builder: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> Default for GameDataBuilder<'a, 'b> {
    fn default() -> Self {
        GameDataBuilder::new()
    }
}

impl<'a, 'b> GameDataBuilder<'a, 'b> {
    /// Create new builder
    pub fn new() -> Self {
        GameDataBuilder {
            dispatcher_operations: Vec::new(),
            disp_builder: DispatcherBuilder::new(),
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
    /// GameDataBuilder::default()
    ///     .with(NopSystem, "tabby cat", &[])
    ///     .with(NopSystem, "tom cat", &[])
    ///     .with_barrier()
    ///     .with(NopSystem, "doggo", &[]);
    /// ~~~
    pub fn with_barrier(mut self) -> Self {
        self.dispatcher_operations.push(Box::new(AddBarrier));
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
    /// GameDataBuilder::default()
    ///     // This will add the "foo" system to the game loop, in this case
    ///     // the "foo" system will not depend on any systems.
    ///     .with(NopSystem, "foo", &[])
    ///     // The "bar" system will only run after the "foo" system has completed
    ///     .with(NopSystem, "bar", &["foo"])
    ///     // It is legal to register a system with an empty name
    ///     .with(NopSystem, "", &[]);
    /// ~~~
    pub fn with<SD, S>(
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
        self.dispatcher_operations.push(dispatcher_operation);
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
    /// GameDataBuilder::default()
    ///     // the Nop system is registered here
    ///     .with_thread_local(NopSystem);
    /// ~~~
    pub fn with_thread_local<SD, S>(mut self, system_desc: SD) -> Self
    where
        SD: SystemDesc<'a, 'b, S> + 'b + 'static,
        S: for<'c> System<'c> + 'static,
    {
        self.dispatcher_operations.push(Box::new(AddThreadLocal {
            system_desc,
            marker: PhantomData::<S>,
        }));
        self
    }

    /// Add a given ECS bundle to the game loop.
    ///
    /// A bundle is a container for registering a bunch of ECS systems at once.
    ///
    /// # Parameters
    ///
    /// - `world`: The `World` that contains all resources.
    /// - `bundle`: The bundle to add.
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
        B: SystemBundle<'a, 'b> + 'static,
    {
        self.dispatcher_operations
            .push(Box::new(AddBundle { bundle }));
        Ok(self)
    }

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
}

impl<'a, 'b> DataInit<GameData<'a, 'b>> for GameDataBuilder<'a, 'b> {
    fn build(self, mut world: &mut World) -> GameData<'a, 'b> {
        #[cfg(not(no_threading))]
        let pool = (*world.read_resource::<ArcThreadPool>()).clone();

        let mut dispatcher_builder = self.disp_builder;

        self.dispatcher_operations
            .into_iter()
            .try_for_each(|dispatcher_operation| {
                dispatcher_operation.exec(world, &mut dispatcher_builder)
            })
            .unwrap_or_else(|e| panic!("Failed to set up dispatcher: {}", e));

        #[cfg(not(no_threading))]
        let mut dispatcher = dispatcher_builder.with_pool(pool).build();
        #[cfg(no_threading)]
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut world);
        GameData::new(dispatcher)
    }
}

impl DataInit<()> for () {
    fn build(self, _: &mut World) {}
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

#[derive(Debug)]
struct AddBarrier;

impl<'a, 'b> DispatcherOperation<'a, 'b> for AddBarrier {
    fn exec(
        self: Box<Self>,
        _world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        dispatcher_builder.add_barrier();
        Ok(())
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
struct AddSystem<SD, S> {
    #[derivative(Debug = "ignore")]
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

#[derive(Derivative)]
#[derivative(Debug)]
struct AddThreadLocal<SD, S> {
    #[derivative(Debug = "ignore")]
    system_desc: SD,
    marker: PhantomData<S>,
}

impl<'a, 'b, SD, S> DispatcherOperation<'a, 'b> for AddThreadLocal<SD, S>
where
    SD: SystemDesc<'a, 'b, S>,
    S: for<'c> System<'c> + 'b,
{
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let system = self.system_desc.build(world);
        dispatcher_builder.add_thread_local(system);
        Ok(())
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
struct AddBundle<B> {
    #[derivative(Debug = "ignore")]
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
