///! Example of a basic entity-component system with 3 types of components.
extern crate time;

extern crate amethyst_ecs as ecs;

use time::Duration;

use ecs::{World, Simulation, Processor, Planner, Component, VecStorage};

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

struct Render;
impl Processor for Render {
    fn process(&mut self, planner: &mut Planner<()>, _: Duration) {
        planner.run0w2r(|p: &Position, _: &Mesh| {
            println!("Render {:?}", p);
        });
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Light>();
    world.register::<Mesh>();

    let mut simulation = Simulation::build(world, 4)
                             .with(Render)
                             .done()
                             .unwrap();

    for i in 0..180 {
        simulation.world().create_now()
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
