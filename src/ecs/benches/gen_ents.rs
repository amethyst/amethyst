#![feature(test)]

extern crate test;
use test::Bencher;

extern crate amethyst_ecs;
use amethyst_ecs::*;

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

#[bench]
fn gen_ents(b: &mut Bencher) {
    b.iter(|| {
        let mut world = World::new();
        
        for _ in 0..180 {
            let ent = world.create_entity();
            world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
            world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
            world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        }
    });
}
