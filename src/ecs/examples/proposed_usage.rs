//! Proposed API usage for when the ECS library is complete.

extern crate amethyst_ecs;
use amethyst_ecs as ecs;

// Define our processors.

struct Rendering;
impl ecs::Processor for Rendering {
    fn process(&mut self) {
        println!("Tick!");
    }
}

// Define our components.

struct Position {
    x: f32,
    y: f32,
    z: f32,
}

fn main () {
    let mut world = ecs::World::new();
    let mut sim = ecs::Simulation::new();

    sim.add_processor(Rendering);

    // Alternatively, use a builder pattern:
    //
    // let (mut sim, _errs) = ecs::Simulation::build()
    //                                        .with(Rendering)
    //                                        .done();

    let ent = world.create_entity();
    world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });

    // TODO: Create entity builder pattern.
    //
    // let ent = world.build_entity()
    //                .with(Position { x: 0.0, y: 0.0, z: 0.0 })
    //                .done();

    for _ in 0..5 {
        // Put game logic here.

        // TODO: Add `Duration` param to `step()` method. Not added because of
        // possible circular dep with `amethyst_engine`, which re-exports the
        // time crate's `Duration` type.
        //
        // let dt = get_delta_time_from_somewhere();
        // world = sim.step(world, dt);
        world = sim.step(world);
    }
}
