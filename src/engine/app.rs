//! The core engine framework.

use num_cpus;
use std::time::{Duration, Instant};

use ecs::{Component, Planner, Priority, System, World};
use ecs::components::{LocalTransform, Transform, Child, Init, Renderable};
use ecs::resources::Time;
use ecs::systems::TransformSystem;
use engine::state::{State, StateMachine};
use engine::timing::Stopwatch;
use gfx_device;
use gfx_device::{DisplayConfig, GfxDevice};
use gfx_device::gfx_types::Factory;
use renderer::{AmbientLight, DirectionalLight, Pipeline, PointLight, target};

/// A context, which stores structs
/// that are required for asset instantiation,
/// like the gfx factory.
pub struct Context {
    /// The gfx factory which is used
    /// for creating buffers.
    pub factory: Factory,
}

/// The engine type, which holds
/// several structs which are needed
/// throughout the whole runtime.
/// These are also used by
/// the user, which allows
/// him to use the `context`
/// or access the world.
pub struct Engine {
    /// The context which is used for
    /// loading assets.
    pub context: Context,
    /// The graphics pipeline
    pub pipe: Pipeline,
    /// The ecs planner
    ///
    /// To get the world, use `world_mut`.
    pub planner: Planner<()>,
}

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    /// The engine of this application which holds
    /// some global structs, like the asset_loader, the
    /// context and gfx structs.
    pub engine: Engine,

    // Graphics and asset management structs.
    // TODO: Refactor so `pipe` and `gfx_device` are moved into the renderer.
    gfx_device: GfxDevice,

    // State management and game loop timing structs.
    delta_time: Duration,
    fixed_step: Duration,
    last_fixed_update: Instant,
    states: StateMachine,
    timer: Stopwatch,
}

impl Context {
    /// Creates a new context. At the moment,
    /// the `Context` only contains a `Factory` for
    /// creating gfx objects.
    ///
    /// This will also be the place for
    /// things like an audio context in the
    /// future.
    pub fn new(factory: Factory) -> Self {
        Context { factory: factory }
    }
}

impl Engine {
    fn new(context: Context, pipe: Pipeline, planner: Planner<()>) -> Self {
        Engine {
            context: context,
            pipe: pipe,
            planner: planner,
        }
    }
}

impl Application {
    /// Creates a new Application with the given initial game state, planner,
    /// and display configuration.
    pub fn new<T>(initial_state: T, mut planner: Planner<()>, cfg: DisplayConfig) -> Application
        where T: State + 'static
    {
        use ecs::resources::{Camera, Projection, ScreenDimensions};

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

        let context = Context::new(factory);

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

        let engine = Engine::new(context, pipe, planner);

        Application {
            engine: engine,
            states: StateMachine::new(initial_state),
            gfx_device: device,
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
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.delta_time = self.timer.elapsed();
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start(&mut self.engine);
    }

    fn should_load_asset(delta_time: f32) -> bool {
        // TODO: don't hardocode
        const FPS: f32 = 60.0;
        const REQUIRED_OVERHANG: f32 = 0.1;
        const REQUIRED_DELTA_TIME: f32 = (1.0 / FPS) * (1.0 - REQUIRED_OVERHANG);

        delta_time < REQUIRED_DELTA_TIME
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        use ecs::resources::ScreenDimensions;

        {
            let events = self.gfx_device.poll_events();
            let events = events.as_ref();

            self.states.handle_events(events, &mut self.engine);

            if self.last_fixed_update.elapsed() >= self.fixed_step {
                self.states.fixed_update(&mut self.engine);
                self.last_fixed_update += self.fixed_step;
            }

            self.states.update(&mut self.engine);
        }

        self.engine.planner.dispatch(());
        self.engine.planner.wait();

        {
            let world = &mut self.engine.planner.mut_world();
            if let Some((w, h)) = self.gfx_device.get_dimensions() {
                let mut dim = world.write_resource::<ScreenDimensions>();
                dim.update(w, h);
            }

            {
                let mut time = world.write_resource::<Time>();
                time.delta_time = self.delta_time;
                time.fixed_step = self.fixed_step;
                time.last_fixed_update = self.last_fixed_update;
            }

            let pipe = &mut self.engine.pipe;
            self.gfx_device.render_world(world, pipe);
        }
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<T>
    where T: State + 'static
{
    config: DisplayConfig,
    initial_state: T,
    planner: Planner<()>,
}

impl<T> ApplicationBuilder<T>
    where T: State + 'static
{
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: DisplayConfig) -> ApplicationBuilder<T> {
        ApplicationBuilder {
            config: cfg,
            initial_state: initial_state,
            planner: Planner::new(World::new(), num_cpus::get()),
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
        Application::new(self.initial_state, self.planner, self.config)
    }
}
