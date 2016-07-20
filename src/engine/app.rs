//! The core engine framework.

use super::state::{State, StateMachine};
use context::timing::{SteadyTime, Stopwatch};
use context::event::EngineEvent;
use context::{Config, Context};

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    states: StateMachine,
    timer: Stopwatch,
    context: Context,
}

impl Application {
    /// Creates a new Application with the given initial game state and a given `Context`.
    pub fn new<T: 'static>(initial_state: T, config: Config) -> Application
        where T: State
    {
        let context = Context::new(config);
        Application {
            states: StateMachine::new(initial_state),
            timer: Stopwatch::new(),
            context: context,
        }
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.context.delta_time = self.timer.elapsed()
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start(&mut self.context);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        let engine_events = self.context.poll_engine_events();
        for engine_event in engine_events {
            self.context.broadcaster.publish().with::<EngineEvent>(engine_event);
        }
        let events = self.context.broadcaster.poll();
        self.states.handle_events(events, &mut self.context);
        while SteadyTime::now() - self.context.last_fixed_update > self.context.fixed_step {
            self.states.fixed_update(&mut self.context);
            // self.systems.fixed_iterate(self.fixed_step);
            self.context.last_fixed_update = self.context.last_fixed_update + self.context.fixed_step;
        }

        self.states.update(&mut self.context);
        // self.systems.iterate(self.delta_time);
        self.context.broadcaster.clean();
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder
    }
}
