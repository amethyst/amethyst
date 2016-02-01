extern crate amethyst_ecs;

use amethyst_ecs::World;

#[allow(dead_code)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

fn main() {
    let mut world = World::new();
    let object = world.create_entity();

    world.insert_component(object, Position { x: 0.0, y: 0.0, z: 0.0 });
    world.destroy_entity(object);

    println!("{:#?}", world);
}
