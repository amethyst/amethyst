extern crate amethyst_ecs;

use amethyst_ecs::*;

#[derive(Debug)]
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
    let rebuilder = Rebuilder::new::<Position, Velocity, _>(move |prev, vel, cur| {
		cur.x = prev.x + vel.x;
		cur.y = prev.y + vel.y;
		cur.z = prev.z + vel.z;
	});
	let pos1 = Position { x: 0.0, y: 0.0, z: 0.0 };
	let vel1 = Velocity { x: 0.3, y: 0.3, z: 0.3 };
	let mut pos2 = Position { x: 0.0, y: 0.0, z: 0.0 };
    rebuilder.rebuild::<Position, Velocity>(&pos1, &vel1, &mut pos2);
    println!("First position: {:?}", pos1);
    println!("Second position: {:?}", pos2);
}
