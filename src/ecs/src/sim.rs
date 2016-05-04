//! Computes the next state.

use time::Duration;

use specs::Planner;

use super::{World, Processor};

pub struct Simulation {
    planner: Planner<Duration>,
}

impl Simulation {
    /// Creates an empty simulation.
    pub fn new(world: World, num_threads: usize) -> Simulation {
        Simulation { planner: Planner::new(world, num_threads) }
    }

    /// Creates an initialized simulation using the [builder pattern][bp].
    ///
    /// [bp]: https://doc.rust-lang.org/book/method-syntax.html#builder-pattern
    pub fn build(world: World, num_threads: usize) -> SimBuilder {
        SimBuilder::new(world, num_threads)
    }

    /// Adds a new processor to the simulation.
    pub fn add_processor<T: Processor<Duration> + 'static>(&mut self,
                                                           p: T,
                                                           name: &str,
                                                           priority: i32) {
        self.planner.add_system(p, name, priority);
    }

    /// Get a reference to the world.
    pub fn world(&self) -> &World {
        &self.planner.world
    }

    /// Computes the next state of the world using the given processors.
    pub fn step(&mut self, dt: Duration) {
        self.planner.dispatch(dt);
        self.planner.wait();
    }
}

/// Consuming builder for easily constructing a new simulations.
pub struct SimBuilder {
    sim: Simulation,
}

impl SimBuilder {
    /// Starts building a new simulation.
    pub fn new(world: World, num_threads: usize) -> SimBuilder {
        SimBuilder { sim: Simulation::new(world, num_threads) }
    }

    /// Add a given processor to the simulation.
    pub fn with<T: Processor<Duration> + 'static>(mut self,
                                                  p: T,
                                                  name: &str,
                                                  priority: i32)
                                                  -> SimBuilder {
        self.sim.add_processor(p, name, priority);
        self
    }

    /// Returns the newly-built simulation or a list of any errors the
    /// processors may have encountered.
    pub fn done(self) -> Simulation {
        self.sim
    }
}
