extern crate amethyst_ecs;

use amethyst_ecs::*;

struct Position {
    x: f32,
    y: f32,
    z: f32,
}

struct Velocity {
	x: f32,
	y: f32,
	z: f32
}

fn main() {
    let mut world = World::new();
    
    for i in 0..180 {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Velocity { x: 0.5, y: 0.5, z: 0.5 });
    }
}
