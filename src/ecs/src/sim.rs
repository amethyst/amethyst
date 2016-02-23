//! Computes the next state.

use processor::{Processor, ProccessorResult};
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
    pub fn add_processor<T: Processor + 'static>(&mut self, p: T) -> ProccessorResult {
        self.procs.push(Box::new(p));
        Ok(())
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
    errors: Vec<String>,
    sim: Simulation,
}

impl SimBuilder {
    /// Starts building a new simulation.
    pub fn new() -> SimBuilder {
        SimBuilder {
            errors: Vec::new(),
            sim: Simulation::new(),
        }
    }

    /// Add a given processor to the simulation.
    pub fn with<T: Processor + 'static>(mut self, p: T) -> SimBuilder {
        let r = self.sim.add_processor(p);
        if let Err(e) = r {
            self.errors.push(e);
        }
        self
    }

    /// Returns the newly-built simulation or a list of any errors the
    /// processors may have encountered.
    pub fn done(self) -> Result<Simulation, Vec<String>> {
        if self.errors.is_empty() {
            Ok(self.sim)
        } else {
            Err(self.errors)
        }
    }
}
