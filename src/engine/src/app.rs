//! The core engine framework.

use super::state::{State, StateMachine};
use super::timing::{Duration, SteadyTime, Stopwatch};

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    delta_time: Duration,
    fixed_step: Duration,
    last_fixed_update: SteadyTime,
    states: StateMachine,
    timer: Stopwatch,
}

impl Application {
    /// Creates a new Application with the given initial game state.
    pub fn new<T: 'static>(initial_state: T) -> Application
        where T: State
    {
        Application {
            delta_time: Duration::zero(),
            fixed_step: Duration::microseconds(16666),
            last_fixed_update: SteadyTime::now(),
            states: StateMachine::new(initial_state),
            timer: Stopwatch::new(),
        }
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.delta_time = self.timer.elapsed()
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start();
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        // self.states.handle_events(&self.event_queue.poll());

        while SteadyTime::now() - self.last_fixed_update > self.fixed_step {
            self.states.fixed_update(self.fixed_step);
            // self.systems.fixed_iterate(self.fixed_step);
            self.last_fixed_update = self.last_fixed_update + self.fixed_step;
        }

        self.states.update(self.delta_time);
        // self.systems.iterate(self.delta_time);
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder
    }
}
