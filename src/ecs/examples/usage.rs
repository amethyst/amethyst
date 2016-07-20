///! Example of a basic entity-component system with 3 types of generic components.
extern crate time;
extern crate rand;

extern crate amethyst_ecs as ecs;

use time::Duration;

use ecs::{World, Simulation, Processor, RunArg, Component, VecStorage, JoinIter};

// First we define our components.

// Position in 3d of the Entity
#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    fn add_speed(&mut self, speed: &Speed) {
        self.x += speed.dx;
        self.y += speed.dy;
        self.z += speed.dz;
    }
}

impl Component for Position {
    type Storage = VecStorage<Position>;
}

// Example of a mesh component
#[derive(Debug)]
struct Mesh {
    handle: u64,
    y: usize,
}
impl Component for Mesh {
    type Storage = VecStorage<Mesh>;
}
// Example of a speed component
#[derive(Debug)]
struct Speed {
    dx: f32,
    dy: f32,
    dz: f32,
}
impl Component for Speed {
    type Storage = VecStorage<Speed>;
}

// Define our processors.

struct Update;
impl Processor<Duration> for Update {
    fn run(&mut self, arg: RunArg, _: Duration) {
        let (mut p, s) = arg.fetch(|w| (w.write::<Position>(), w.read::<Speed>())); // Make p writable.
        for (p, s) in JoinIter::new((&mut p, &s)) {
            // We want to only update entities with position and speed.
            p.add_speed(&s);
        }
    }
}

struct Render {
    frame_count: u32,
}
impl Processor<Duration> for Render {
    fn run(&mut self, arg: RunArg, _: Duration) {
        let (p, m) = arg.fetch(|w| (w.read::<Position>(), w.read::<Mesh>())); // Make p writable.
        for (p, _) in JoinIter::new((&p, &m)) {
            // We want to only render entities with position and mesh.
            println!("Render {:?}", p);
        }
        self.frame_count += 1;
        println!("Frames: {}", self.frame_count);
    }
}

fn main() {
    use rand::distributions::{IndependentSample, Range};
    let mut rng = rand::thread_rng();
    let between = Range::new(-10f32, 10.);

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Speed>();
    world.register::<Mesh>();

    let mut simulation = Simulation::build(world, 4)
                             .with(Update, "Updater", 1000) // High priority
                             .with(Render { frame_count: 0 }, "Renderer", 500) // Low priority
                             .done();

    for _ in 0..180 {
        simulation.mut_world()
                  .create_now()
                  .with(Position {
                      x: 0.0,
                      y: 0.0,
                      z: 0.0,
                  })
                  .with(Mesh {
                      handle: 1234567890,
                      y: 12,
                  })
                  .with(Speed {
                      dx: between.ind_sample(&mut rng),
                      dy: between.ind_sample(&mut rng),
                      dz: between.ind_sample(&mut rng),
                  })
                  .build();
    }

    for _ in 0..3 {
        // Put game logic here.

        simulation.step(Duration::milliseconds(3));
    }
}
