extern crate amethyst_ecs;

use amethyst_ecs::*;

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

struct Light {
    x: f32,
    y: f32,
    z: f32,
}

struct Mesh {
    handle: u64,
    y: usize,
}

fn main() {
    let mut world = World::new();
    
    for i in 0..180 {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: i as f32 * 0.1, y: 0.0, z: 0.0 });
        world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
    }
    println!("Component Position #{}: {:?}", 60, world.get_component::<Position>(60).unwrap());
}
