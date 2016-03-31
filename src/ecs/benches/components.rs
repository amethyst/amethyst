#![feature(test)]
#![feature(step_by)]

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
fn insert(b: &mut Bencher) {
    let mut world = World::new();

    let mut ents = Vec::with_capacity(2000);
    for _ in 0..ents.capacity() {
        ents.push(world.create_entity());
    }
    b.iter(|| {
        for i in 0..ents.len() {
            let ent = ents.get(i).unwrap().clone();
            world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
            world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
            world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        }
    });
}

#[bench]
fn get_one(b: &mut Bencher) {
    let mut world = World::new();

    let mut ents = Vec::with_capacity(2000);
    for _ in 0..ents.capacity() {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        ents.push(ent);
    }
    b.iter(|| {
        for i in 0..ents.len() {
            test::black_box(world.component::<Position>(i));
        }
    });
}

#[bench]
fn get_multiple(b: &mut Bencher) {
    let mut world = World::new();

    let mut ents = Vec::with_capacity(2000);
    for _ in 0..ents.capacity() {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        ents.push(ent);
    }
    b.iter(|| {
        for i in 0..ents.len() {
            test::black_box(world.component::<Position>(i));
            test::black_box(world.component::<Light>(i));
            test::black_box(world.component::<Mesh>(i));
        }
    });
}

#[bench]
fn get_step_large_one(b: &mut Bencher) {
    let mut world = World::new();

    let mut ents = Vec::with_capacity(2000);
    for _ in 0..ents.capacity() {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        ents.push(ent);
    }
    b.iter(|| {
        for i in (0..ents.len()).step_by(15) {
            test::black_box(world.component::<Position>(i));
        }
    });
}

#[bench]
fn get_step_large_multiple(b: &mut Bencher) {
    let mut world = World::new();

    let mut ents = Vec::with_capacity(2000);
    for _ in 0..ents.capacity() {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        ents.push(ent);
    }
    b.iter(|| {
        for i in (0..ents.len()).step_by(15) {
            test::black_box(world.component::<Position>(i));
            test::black_box(world.component::<Light>(i));
            test::black_box(world.component::<Mesh>(i));
        }
    });
}

#[bench]
fn get_step_small_one(b: &mut Bencher) {
    let mut world = World::new();

    let mut ents = Vec::with_capacity(2000);
    for _ in 0..ents.capacity() {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        ents.push(ent);
    }
    b.iter(|| {
        for i in (0..ents.len()).step_by(3) {
            test::black_box(world.component::<Position>(i));
        }
    });
}

#[bench]
fn get_step_small_multiple(b: &mut Bencher) {
    let mut world = World::new();

    let mut ents = Vec::with_capacity(2000);
    for _ in 0..ents.capacity() {
        let ent = world.create_entity();
        world.insert_component(ent, Position { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Light { x: 0.0, y: 0.0, z: 0.0 });
        world.insert_component(ent, Mesh { handle: 1234567890, y: 12 });
        ents.push(ent);
    }
    b.iter(|| {
        for i in (0..ents.len()).step_by(3) {
            test::black_box(world.component::<Position>(i));
            test::black_box(world.component::<Light>(i));
            test::black_box(world.component::<Mesh>(i));
        }
    });
}
