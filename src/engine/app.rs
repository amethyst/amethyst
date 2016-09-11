//! The core engine framework.

use super::state::{State, StateMachine};
use context::timing::Stopwatch;
use context::event::EngineEvent;
use context::Context;
use ecs::{Planner, World, Processor, Priority, Component};
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    states: StateMachine,
    timer: Stopwatch,
    context: Arc<Mutex<Context>>,
}

impl Application {
    /// Creates a new Application with the given initial game state, planner, and context.
    pub fn new<T>(initial_state: T,
                  planner: Planner<Arc<Mutex<Context>>>,
                  ctx: Context)
                  -> Application
        where T: State + 'static
    {
        let context = Arc::new(Mutex::new(ctx));
        Application {
            states: StateMachine::new(initial_state, planner),
            timer: Stopwatch::new(),
            context: context,
        }
    }

    /// Build a new Application using builder pattern.
    pub fn build<T>(initial_state: T, ctx: Context) -> ApplicationBuilder<T>
        where T: State + 'static
    {
        ApplicationBuilder::new(initial_state, ctx)
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.context.lock().unwrap().delta_time = self.timer.elapsed();
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start(self.context.lock().unwrap().deref_mut());
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        let events = self.context.lock().unwrap().poll_engine_events();
        for e in events {
            self.context.lock().unwrap().broadcaster.publish().with::<EngineEvent>(e);
        }

        let entities = self.context.lock().unwrap().broadcaster.poll();
        self.states.handle_events(&entities, self.context.lock().unwrap().deref_mut());

        let fixed_step = self.context.lock().unwrap().fixed_step;
        let last_fixed_update = self.context.lock().unwrap().last_fixed_update;

        if last_fixed_update.elapsed() >= fixed_step {
            self.states.fixed_update(self.context.lock().unwrap().deref_mut());
            self.context.lock().unwrap().last_fixed_update += fixed_step;
        }

        self.states.update(self.context.lock().unwrap().deref_mut());
        self.states.run_processors(self.context.clone());
        self.context.lock().unwrap().renderer.submit();
        self.context.lock().unwrap().broadcaster.clean();
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
    initial_state: T,
    context: Context,
    planner: Planner<Arc<Mutex<Context>>>,
}

impl<T> ApplicationBuilder<T>
    where T: State + 'static
{
    pub fn new(initial_state: T, ctx: Context) -> ApplicationBuilder<T> {
        let world = World::new();
        ApplicationBuilder {
            initial_state: initial_state,
            context: ctx,
            planner: Planner::new(world, 1),
        }
    }

    pub fn register<C>(mut self) -> ApplicationBuilder<T>
        where C: Component
    {
        {
            let world = &mut self.planner.mut_world();
            world.register::<C>();
        }
        self
    }

    pub fn with<P>(mut self, pro: P, name: &str, pri: Priority) -> ApplicationBuilder<T>
        where P: Processor<Arc<Mutex<Context>>> + 'static
    {
        self.planner.add_system::<P>(pro, name, pri);
        self
    }

    pub fn done(self) -> Application {
        Application::new(self.initial_state, self.planner, self.context)
    }
}
