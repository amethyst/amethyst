///! Example of a basic entity-component system with 3 types of components.
extern crate time;

extern crate amethyst_ecs as ecs;

use time::Duration;

use ecs::{World, Simulation, Processor, RunArg, Component, VecStorage, JoinIter};

// Define our components.

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}
impl Component for Position {
    type Storage = VecStorage<Position>;
}

#[derive(Debug)]
struct Light {
    x: f32,
    y: f32,
    z: f32,
}
impl Component for Light {
    type Storage = VecStorage<Light>;
}

#[derive(Debug)]
struct Mesh {
    handle: u64,
    y: usize,
}
impl Component for Mesh {
    type Storage = VecStorage<Mesh>;
}

// Define our processors.

struct Render {
    frame_count: u32,
}
impl Processor<Duration> for Render {
    fn run(&mut self, arg: RunArg, _: Duration) {
        let (p, m) = arg.fetch(|w| (w.read::<Position>(), w.read::<Mesh>()));
        for (p, _) in JoinIter::new((&p, &m)) {
            println!("Render {:?}", p);
        }
        self.frame_count += 1;
        println!("Frames: {}", self.frame_count);
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Light>();
    world.register::<Mesh>();

    let mut simulation = Simulation::build(world, 4)
                             .with(Render { frame_count: 0 }, "Renderer", 1000)
                             .done();

    for i in 0..180 {
        simulation.world()
                  .create_now()
                  .with(Position {
                      x: i as f32 * 0.1,
                      y: 0.0,
                      z: 0.0,
                  })
                  .with(Light {
                      x: 0.0,
                      y: 0.0,
                      z: 0.0,
                  })
                  .with(Mesh {
                      handle: 1234567890,
                      y: 12,
                  })
                  .build();
    }

    for _ in 0..3 {
        // Put game logic here.

        simulation.step(Duration::milliseconds(3));
    }
}
