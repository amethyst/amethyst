use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use crate::{
    core::{
        specs::prelude::{Dispatcher, DispatcherBuilder, RunNow, System, World},
        ArcThreadPool, SimpleDispatcherBuilder, SystemBundle,
    },
    error::{Error, Result},
    renderer::pipe::pass::Pass,
};

/// Initialise trait for game data
pub trait DataInit<T> {
    /// Build game data
    fn build(self, world: &mut World) -> T;
}

/// Default game data
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

/// Implementation detail
///
/// See `GameDataBuilder::commands`
trait BuildGameData<'a, 'b> {
    fn build(self: Box<Self>, disp_builder: &mut DispatcherBuilder<'a, 'b>);
}

/// Automatic dependencies for system
///
/// See `GameDataBuilder::with_auto` for more details.
pub trait AutoAddSystem: for<'a> System<'a> {
    /// System dependencies
    ///
    /// See `GameDataBuilder::with_auto` for more details.
    const DEPENDENCIES: &'static [&'static str];
    /// System reverse dependencies
    ///
    /// See `GameDataBuilder::with_auto` for more details.
    const REVERSE_DEPENDENCIES: &'static [&'static str];
}

/// Builder for default game data
pub struct GameDataBuilder<'a, 'b, 'c> {
    disp_builder: DispatcherBuilder<'a, 'b>,
    commands: Vec<Box<dyn BuildGameData<'a, 'b> + 'c>>,
    dependencies: HashMap<&'c str, Rc<RefCell<Vec<&'c str>>>>,
    added_names: HashSet<&'c str>,
}

impl<'a, 'b, 'c> Default for GameDataBuilder<'a, 'b, 'c> {
    fn default() -> Self {
        GameDataBuilder::new()
    }
}

impl<'a, 'b, 'c> GameDataBuilder<'a, 'b, 'c> {
    /// Create new builder
    pub fn new() -> Self {
        GameDataBuilder {
            disp_builder: DispatcherBuilder::new(),
            commands: Vec::new(),
            dependencies: HashMap::new(),
            added_names: HashSet::new(),
        }
    }

    /// Inserts a barrier which assures that all systems added before the
    /// barrier are executed before the ones after this barrier.
    ///
    /// Does nothing if there were no systems added since the last call to
    /// `with_barrier()` or `add_barrier()`. Thread-local systems are not affected by barriers;
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
    ///     .with(NopSystem, "tabby cat", &[], &[])
    ///     .with(NopSystem, "tom cat", &[], &[])
    ///     .with_barrier()
    ///     .with(NopSystem, "doggo", &[], &[]);
    /// ~~~
    pub fn with_barrier(mut self) -> Self {
        self.add_barrier();
        self
    }

    /// Inserts a barrier which assures that all systems added before the
    /// barrier are executed before the ones after this barrier.
    ///
    /// Does nothing if there were no systems added since the last call to
    /// `with_barrier()` or `add_barrier()`. Thread-local systems are not affected by barriers;
    /// they're always executed at the end.
    ///
    /// See `with_barrier` for example.
    pub fn add_barrier(&mut self) {
        struct AddBarrier;

        impl<'a, 'b> BuildGameData<'a, 'b> for AddBarrier {
            fn build(self: Box<Self>, disp_builder: &mut DispatcherBuilder<'a, 'b>) {
                disp_builder.add_barrier();
            }
        }

        self.commands.push(Box::new(AddBarrier));
    }

    /// Adds a given system.
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
    /// - `reverse_dependencies`: A list of named system that _must not_ start running
    ///                         before this system have completed running.
    ///                         This may be an empty list if there is no dependencies.
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
    ///     .with(NopSystem, "foo", &[], &[])
    ///     // The "bar" system will only run after the "foo" system has completed
    ///     .with(NopSystem, "bar", &["foo"], &[])
    ///     // The "baz" system will only run after the "foo" system has completed
    ///     // and the "bar" system will run only after "baz" system has completed
    ///     .with(NopSystem, "baz", &["foo"], &["bar"])
    ///     // It is legal to register a system with an empty name
    ///     .with(NopSystem, "", &[], &[]);
    /// ~~~
    pub fn with<'i, S>(
        mut self,
        system: S,
        name: &'c str,
        dependencies: impl IntoIterator<Item = &'i &'c str>,
        reverse_dependencies: impl IntoIterator<Item = &'i &'c str>,
    ) -> Self
    where
        for<'d> S: System<'d> + Send + 'a + 'c,
        'c: 'i,
    {
        self.add(system, name, dependencies, reverse_dependencies);
        self
    }

    /// Adds a given system with automatic dependencies
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
    /// - `reverse_dependencies`: A list of named system that _must not_ start running
    ///                         before this system have completed running.
    ///                         This may be an empty list if there is no dependencies.
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
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::prelude::System;
    /// use amethyst::AutoAddSystem;
    ///
    /// struct NopSystem;
    /// impl<'a> System<'a> for NopSystem {
    ///     type SystemData = ();
    ///     fn run(&mut self, _: Self::SystemData) {}
    /// }
    ///
    /// impl AutoAddSystem for NopSystem {
    ///     // If this system is added using GameDataBuilder::with_auto it will only run
    ///     // after the "foo" system has completed
    ///     const DEPENDENCIES: &'static [&'static str] = &["foo"];
    ///     // and the "bar" system will run only after this system has completed
    ///    const REVERSE_DEPENDENCIES: &'static [&'static str] = &["bar"];
    /// }
    ///
    /// GameDataBuilder::default()
    ///     // This will add the "foo" system to the game loop, in this case
    ///     // the "foo" system will not depend on any systems.
    ///     .with(NopSystem, "foo", &[], &[])
    ///     // The "bar" system will only run after the "foo" system has completed
    ///     .with(NopSystem, "bar", &["foo"], &[])
    ///     // Adds `NopSystem` with automatic dependencies
    ///     .with_auto(NopSystem, "baz")
    ///     // It is legal to register a system with an empty name
    ///     .with(NopSystem, "", &[], &[]);
    /// ~~~
    pub fn with_auto<S>(mut self, system: S, name: &'c str) -> Self
    where
        for<'d> S: System<'d> + Send + 'a + 'c,
        S: AutoAddSystem,
    {
        self.add(system, name, S::DEPENDENCIES, S::REVERSE_DEPENDENCIES);
        self
    }

    /// Adds a given system.
    ///
    /// See `with` for example.
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
    ///                 This may be an empty list if there is no dependencies.
    /// - `reverse_dependencies`: A list of named system that _must not_ start running
    ///                         before this system have completed running.
    ///                         This may be an empty list if there is no dependencies.
    ///
    /// # Type Parameters
    ///
    /// - `S`: A type that implements the `System` trait.
    ///
    /// # Panics
    ///
    /// If two system are added that share an identical name, this function will panic.
    /// Empty names are permitted, and this function will not panic if more then two are added.
    pub fn add<'i, S>(
        &mut self,
        system: S,
        name: &'c str,
        dependencies: impl IntoIterator<Item = &'i &'c str>,
        reverse_dependencies: impl IntoIterator<Item = &'i &'c str>,
    ) where
        for<'d> S: System<'d> + Send + 'a + 'c,
        'c: 'i,
    {
        struct AddSystem<'c, S> {
            system: S,
            name: &'c str,
            dependencies: Rc<RefCell<Vec<&'c str>>>,
        }

        impl<'a, 'b, 'c, S> BuildGameData<'a, 'b> for AddSystem<'c, S>
        where
            for<'d> S: System<'d> + Send + 'a,
        {
            fn build(self: Box<Self>, disp_builder: &mut DispatcherBuilder<'a, 'b>) {
                let dependencies = self.dependencies.clone();
                let AddSystem { name, system, .. } = *self;
                disp_builder.add(system, name, &dependencies.borrow());
            }
        }

        if name != "" && !self.added_names.insert(name) {
            panic!("multiple systems with name `{}`", name);
        }

        let dependencies = dependencies.into_iter().map(|d| *d);

        let dependencies = if name == "" {
            Rc::new(RefCell::new(dependencies.collect()))
        } else {
            let deps = self
                .dependencies
                .entry(name)
                .or_insert_with(|| Rc::new(RefCell::new(Vec::new())));
            deps.borrow_mut().extend(dependencies);
            deps.clone()
        };

        self.commands.push(Box::new(AddSystem {
            system,
            name,
            dependencies,
        }));

        for reverse_dependency in reverse_dependencies {
            self.dependencies
                .entry(*reverse_dependency)
                .and_modify(|deps| deps.borrow_mut().push(name))
                .or_insert_with(|| Rc::new(RefCell::new(vec![name].into())));
        }
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
    /// - `S`: A type that implements the `RunNow` trait. This trait is implemented for all `System`s.
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
    pub fn with_thread_local<S>(mut self, system: S) -> Self
    where
        for<'d> S: RunNow<'d> + 'b + 'c,
    {
        self.add_thread_local(system);
        self
    }

    /// Add a given thread-local system.
    ///
    /// See `with_thread_local` for example.
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
    /// # Type Parameters
    ///
    /// - `S`: A type that implements the `RunNow` trait. This trait is implemented for all `System`s.
    pub fn add_thread_local<S>(&mut self, system: S)
    where
        for<'d> S: RunNow<'d> + 'b + 'c,
    {
        struct AddThreadLocal<S> {
            system: S,
        }

        impl<'a, 'b, S> BuildGameData<'a, 'b> for AddThreadLocal<S>
        where
            for<'d> S: RunNow<'d> + 'b,
        {
            fn build(self: Box<Self>, disp_builder: &mut DispatcherBuilder<'a, 'b>) {
                disp_builder.add_thread_local(self.system);
            }
        }

        self.commands.push(Box::new(AddThreadLocal { system }));
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
    pub fn with_bundle<B>(mut self, bundle: B) -> Result<Self>
    where
        B: SystemBundle<'a, 'b, 'c, Self>,
    {
        bundle.build(&mut self).map_err(Error::Core)?;
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
    pub fn with_basic_renderer<A, P>(self, path: A, pass: P, with_ui: bool) -> Result<Self>
    where
        A: AsRef<Path>,
        P: Pass + 'b + 'c,
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
            self.with_bundle(RenderBundle::new(pipe, Some(config)))
        } else {
            let pipe = Pipeline::build().with_stage(
                Stage::with_backbuffer()
                    .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                    .with_pass(pass),
            );
            self.with_bundle(RenderBundle::new(pipe, Some(config)))
        }
    }
}

impl<'a, 'b, 'c> SimpleDispatcherBuilder<'a, 'b, 'c> for GameDataBuilder<'a, 'b, 'c> {
    fn add<T>(&mut self, system: T, name: &'c str, dep: &[&'c str])
    where
        T: for<'d> System<'d> + Send + 'a + 'c,
    {
        GameDataBuilder::add(self, system, name, dep, &[]);
    }

    fn add_thread_local<T>(&mut self, system: T)
    where
        T: for<'d> RunNow<'d> + 'b + 'c,
    {
        GameDataBuilder::add_thread_local(self, system);
    }

    fn add_barrier(&mut self) {
        GameDataBuilder::add_barrier(self);
    }
}

impl<'a, 'b, 'c> DataInit<GameData<'a, 'b>> for GameDataBuilder<'a, 'b, 'c> {
    fn build(mut self, world: &mut World) -> GameData<'a, 'b> {
        if cfg!(debug_assertions) {
            let unresolved_dependencies = self
                .dependencies
                .keys()
                .filter(|name| !self.added_names.contains(*name))
                .collect::<Vec<_>>();
            assert_eq!(
                unresolved_dependencies.len(),
                0,
                "unresolved dependencies: {:?}",
                unresolved_dependencies
            );
        }

        for command in self.commands.drain(..) {
            command.build(&mut self.disp_builder);
        }

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
