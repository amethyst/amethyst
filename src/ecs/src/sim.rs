//! Computes the next state.

use processor::{Processor, ProcessorError};
use world::World;

pub struct Simulation {
    procs: Vec<Box<Processor>>,
}

impl Simulation {
    /// Creates an empty simulation.
    pub fn new() -> Simulation {
        Simulation { procs: Vec::new() }
    }

    /// Creates an initialized simulation using the [builder pattern][bp].
    ///
    /// [bp]: https://doc.rust-lang.org/book/method-syntax.html#builder-pattern
    pub fn build() -> SimBuilder {
        SimBuilder::new()
    }

    /// Adds a new processor to the simulation.
    pub fn add_processor<T: Processor + 'static>(&mut self, p: T) -> ProcessorError {
        self.procs.push(Box::new(p));
        ProcessorError
    }

    /// Computes the next state of the world using the given processors.
    pub fn step(&mut self, world: World) -> World {
        let mut next_state = world;

        // TODO: Rich possibilities for multithreading here.
        for p in self.procs.iter_mut() {
            p.process();
        }

        next_state
    }
}

/// Consuming builder for easily constructing a new simulations.
pub struct SimBuilder {
    results: Vec<ProcessorError>,
    sim: Simulation,
}

impl SimBuilder {
    /// Starts building a new simulation.
    pub fn new() -> SimBuilder {
        SimBuilder {
            results: Vec::new(),
            sim: Simulation::new(),
        }
    }

    /// Add a given processor to the simulation.
    pub fn with<T: Processor + 'static>(mut self, p: T) -> SimBuilder {
        let r = self.sim.add_processor(p);
        self.results.push(r);
        self
    }

    /// Returns the newly-built simulation plus a stack of any errors the
    /// processors may have encountered.
    pub fn done(self) -> (Simulation, Vec<ProcessorError>) {
        (self.sim, self.results)
    }
}
