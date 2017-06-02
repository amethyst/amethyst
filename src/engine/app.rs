//! The core engine framework.

use std::sync::Arc;
use std::time::{Duration, Instant};

use threadpool::ThreadPool;
#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};
use num_cpus;

use asset_manager::AssetManager;
use ecs::{Component, Planner, Priority, System, World};
use ecs::components::{LocalTransform, Transform, Child, Init, Renderable};
use ecs::resources::Time;
use ecs::systems::TransformSystem;
use engine::state::{State, StateMachine};
use engine::timing::Stopwatch;
use gfx_device;
use gfx_device::{DisplayConfig, GfxDevice, gfx_types};
use renderer::{AmbientLight, DirectionalLight, Pipeline, PointLight, target};

/// The engine type, which holds
/// several useful structs which can be accessed
/// by engine and game.
/// It allows for sharing a thread pool,
/// and it holds the asset manager.
pub struct Engine {
    /// The graphics pipeline
    pub pipe: Pipeline,
    /// The ecs planner
    ///
    /// To get the world, use `world_mut`.
    pub planner: Planner<()>,
    /// The asset manager used to submit
    /// assets to be loaded in parallel
    pub manager: AssetManager,
    /// A thread pool which is used
    /// for asset loading and the ecs.
    pub pool: Arc<ThreadPool>,

    gfx_device: GfxDevice,
}

impl Engine {
    fn new(pipe: Pipeline, planner: Planner<()>,
           manager: AssetManager, pool: Arc<ThreadPool>,
           device: GfxDevice) -> Self {
        Engine {
            pipe: pipe,
            planner: planner,
            manager: manager,
            pool: pool,
            gfx_device: device,
        }
    }

    fn advance_frame(&mut self, delta: Duration, fixed: Duration, last_fixed: Instant) {
        use ecs::resources::ScreenDimensions;

        #[cfg(feature="profiler")]
        profile_scope!("dispatch");
        self.planner.dispatch(());
        self.planner.wait();

        #[cfg(feature="profiler")]
        profile_scope!("render_world");
        {
            use ecs::Gate;

            let world = &mut self.planner.mut_world();
            if let Some((w, h)) = self.gfx_device.get_dimensions() {
                let mut dim = world.write_resource::<ScreenDimensions>().pass();
                dim.update(w, h);
            }

            {
                let mut time = world.write_resource::<Time>().pass();
                time.delta_time = delta;
                time.fixed_step = fixed;
                time.last_fixed_update = last_fixed;
            }

            let pipe = &mut self.pipe;
            self.gfx_device.render_world(world, pipe);
        }
    }
}

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    /// Holds "global" resources per engine.
    pub engine: Engine,

    // State management and game loop timing structs.
    delta_time: Duration,
    fixed_step: Duration,
    last_fixed_update: Instant,
    states: StateMachine,
    timer: Stopwatch,
}

impl Application {
    /// Creates a new Application with the given initial game state, planner,
    /// and display configuration.
    pub fn new<T>(initial_state: T, mut planner: Planner<()>,
                  cfg: DisplayConfig, pool: Arc<ThreadPool>) -> Application
        where T: State + 'static
    {
        use ecs::resources::{Camera, Projection, ScreenDimensions};

        #[cfg(feature = "profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("video_init");
        let (device, mut factory, main_target) = gfx_device::video_init(&cfg);
        let mut pipe = Pipeline::new();
        pipe.targets.insert("main".into(),
                            Box::new(target::ColorBuffer {
                                color: main_target.color.clone(),
                                output_depth: main_target.depth.clone(),
                            }));

        let (w, h) = device.get_dimensions().unwrap();
        let geom_buf = target::GeometryBuffer::new(&mut factory, (w as u16, h as u16));
        pipe.targets.insert("gbuffer".into(), Box::new(geom_buf));

        let mut assets = AssetManager::new();
        assets.add_loader::<gfx_types::Factory>(factory);

        let trans_sys = TransformSystem::new();
        planner.add_system::<TransformSystem>(trans_sys, "transform_system", 0);

        {
            let mut world = planner.mut_world();
            let time = Time {
                delta_time: Duration::new(0, 0),
                fixed_step: Duration::new(0, 16666666),
                last_fixed_update: Instant::now(),
            };
            if let Some((w, h)) = device.get_dimensions() {
                let dim = ScreenDimensions::new(w, h);
                let proj = Projection::Perspective {
                    fov: 90.0,
                    aspect_ratio: dim.aspect_ratio,
                    near: 0.1,
                    far: 100.0,
                };
                let eye = [0.0, 0.0, 0.0];
                let target = [1.0, 0.0, 0.0];
                let up = [0.0, 1.0, 0.0];
                let camera = Camera::new(proj, eye, target, up);
                world.add_resource::<ScreenDimensions>(dim);
                world.add_resource::<Camera>(camera);
            }

            world.add_resource::<AmbientLight>(AmbientLight::default());
            world.add_resource::<Time>(time);
            world.register::<Child>();
            world.register::<DirectionalLight>();
            world.register::<Init>();
            world.register::<LocalTransform>();
            world.register::<PointLight>();
            world.register::<Renderable>();
            world.register::<Transform>();
        }

        let engine = Engine::new(pipe, planner, assets, pool, device);

        Application {
            engine: engine,

            states: StateMachine::new(initial_state),
            timer: Stopwatch::new(),
            delta_time: Duration::new(0, 0),
            fixed_step: Duration::new(0, 16666666),
            last_fixed_update: Instant::now(),
        }
    }

    /// Builds a new application using builder pattern.
    pub fn build<T>(initial_state: T, cfg: DisplayConfig) -> ApplicationBuilder<T>
        where T: State + 'static
    {
        ApplicationBuilder::new(initial_state, cfg)
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        {
            #[cfg(feature="profiler")]
            profile_scope!("initialize");
            self.initialize();
        }

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.delta_time = self.timer.elapsed();
        }

        {
            #[cfg(feature="profiler")]
            profile_scope!("shutdown");
            self.shutdown();
        }
        #[cfg(feature="profiler")]
        self.write_profile();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start(&mut self.engine);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        #[cfg(feature = "profiler")]
        profile_scope!("handle_events");
        let events = self.engine.gfx_device.poll_events();
        self.states.handle_events(events.as_ref(), &mut self.engine);

        #[cfg(feature = "profiler")]
        profile_scope!("fixed_update");
        if self.last_fixed_update.elapsed() >= self.fixed_step {
            self.states.fixed_update(&mut self.engine);
            self.last_fixed_update += self.fixed_step;
        }

        #[cfg(feature = "profiler")]
        profile_scope!("update");
        self.states.update(&mut self.engine);

        self.engine.advance_frame(self.delta_time, self.fixed_step, self.last_fixed_update);
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder.
    }

    #[cfg(feature="profiler")]
    /// Writes thread_profiler profile.
    fn write_profile(&self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json",
                           env!("CARGO_MANIFEST_DIR"));
        write_profile(path.as_str());
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<T>
    where T: State + 'static
{
    config: DisplayConfig,
    initial_state: T,
    planner: Planner<()>,
    pool: Arc<ThreadPool>,
}

impl<T> ApplicationBuilder<T>
    where T: State + 'static
{
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: DisplayConfig) -> ApplicationBuilder<T> {
        let pool = Arc::new(ThreadPool::new(num_cpus::get()));

        ApplicationBuilder {
            config: cfg,
            initial_state: initial_state,
            planner: Planner::from_pool(World::new(), pool.clone()),
            pool: pool,
        }
    }

    /// Registers a given component type.
    pub fn register<C>(mut self) -> ApplicationBuilder<T>
        where C: Component
    {
        {
            let world = &mut self.planner.mut_world();
            world.register::<C>();
        }
        self
    }

    /// Adds a given system `pro`, assigns it the string identifier `name`,
    /// and marks it with the runtime priority `pri`.
    pub fn with<S>(mut self, sys: S, name: &str, pri: Priority) -> ApplicationBuilder<T>
        where S: System<()> + 'static
    {
        self.planner.add_system::<S>(sys, name, pri);
        self
    }

    /// Builds the Application and returns the result.
    pub fn done(self) -> Application {
        Application::new(self.initial_state, self.planner, self.config, self.pool)
    }
}
