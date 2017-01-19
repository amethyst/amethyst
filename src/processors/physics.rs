//! Physics processor based on `nphysics3d`

extern crate ncollide;
extern crate nphysics3d;

use std::ops::{Deref, DerefMut};

use components::physics::PhysicsComponent;
use components::transform::LocalTransform;
use world_resources::Time;

use ecs::{Join, Processor, RunArg};
use self::nphysics3d::world::World;


/// Wrapper around `nphysics`'s `World` that impls `Send`/`Sync`, so that
/// it can be processed by the ECS system. We take care of ensuring that
/// there are no data races inside the ECS system, but the compiler can't
/// infer this, so we have to manually tell it that it's ok.
pub struct PhysicsWorld(World<f32>);

impl PhysicsWorld {
    pub fn new() -> PhysicsWorld {
        PhysicsWorld(World::new())
    }
}

unsafe impl Sync for PhysicsWorld { }
unsafe impl Send for PhysicsWorld { }

impl Deref for PhysicsWorld {
    type Target = World<f32>;
    fn deref(&self) -> &World<f32> {
        &self.0
    }
}

impl DerefMut for PhysicsWorld {
    fn deref_mut(&mut self) -> &mut World<f32> {
        &mut self.0
    }
}


/// Handles physics processing for ECS
pub struct PhysicsProcessor;

impl PhysicsProcessor {
    pub fn new() -> PhysicsProcessor {
        PhysicsProcessor { }
    }
}

impl Processor<()> for PhysicsProcessor {
    fn run(&mut self, arg: RunArg, _: ()) {
        // Grab references to world objects. Note that we need to use
        // `w.write` for the `PhysicsComponent`, as `world.step` below
        // modifies each `PhysicsComponent`, but isn't thread-safe by
        // itself.
        let (
            mut world,
            mut handles,
            mut locals,
            time,
        ) = arg.fetch(|w| (
            w.write_resource::<PhysicsWorld>(),
            w.write::<PhysicsComponent>(),
            w.write::<LocalTransform>(),
            w.read_resource::<Time>(),
        ));

        // Update physics world state
        world.step(time.delta_time.subsec_nanos() as f32 / 1.0e9);

        // Update the `LocalTransform` for each entity that has a
        // physics component.
        for (handle, local) in (&mut handles, &mut locals).iter() {
            let body = handle.borrow();
            let position = body.position();

            // Update translation
            local.translation[0] = position.translation[0];
            local.translation[1] = position.translation[1];
            local.translation[2] = position.translation[2];

            // Update rotation. `nalgebra` stores rotation as a rotation matrix,
            // and `amethyst` expects a quaternion, so the math below converts between
            // the two. See here for a reference on the math:
            // http://www.euclideanspace.com/maths/geometry/rotations/conversions/matrixToQuaternion/
            // See here for thread about having `nalgebra` doing the conversion work:
            // http://users.nphysics.org/t/converting-rotation3-to-quaternion/48
            let rot = position.rotation;
            let trace = rot[(0, 0)] + rot[(1, 1)] + rot[(2, 2)];

            if 0.0 < trace {
              let sqrt = (1.0 + trace).sqrt() * 2.0;
              local.rotation[0] = 0.25 * sqrt;
              local.rotation[1] = (rot[(2, 1)] - rot[(1, 2)]) / sqrt;
              local.rotation[2] = (rot[(0, 2)] - rot[(2, 0)]) / sqrt;
              local.rotation[3] = (rot[(1, 0)] - rot[(0, 1)]) / sqrt;

            } else if (rot[(0, 0)] > rot[(1, 1)]) && (rot[(0, 0)] > rot[(2, 2)]) {
              let sqrt = (1.0 + rot[(0, 0)] - rot[(1, 1)] - rot[(2, 2)]).sqrt() * 2.0;
              local.rotation[0] = (rot[(2, 1)] - rot[(1, 2)]) / sqrt;
              local.rotation[1] = 0.25 * sqrt;
              local.rotation[2] = (rot[(0, 1)] + rot[(1, 0)]) / sqrt;
              local.rotation[3] = (rot[(0, 2)] + rot[(2, 0)]) / sqrt

            } else if rot[(1, 1)] > rot[(2, 2)] {
              let sqrt = (1.0 + rot[(1, 1)] - rot[(0, 0)] - rot[(2, 2)]).sqrt() * 2.0;
              local.rotation[0] = (rot[(0, 2)] - rot[(2, 0)]) / sqrt;
              local.rotation[1] = (rot[(0, 1)] + rot[(1, 0)]) / sqrt;
              local.rotation[2] = 0.25 * sqrt;
              local.rotation[3] = (rot[(1, 2)] + rot[(2, 1)]) / sqrt;

            } else {
              let sqrt = (1.0 + rot[(2, 2)] - rot[(0, 0)] - rot[(1, 1)]).sqrt() * 2.0;
              local.rotation[0] = (rot[(1, 0)] - rot[(0, 1)]) / sqrt;
              local.rotation[1] = (rot[(0, 2)] + rot[(2, 0)]) / sqrt;
              local.rotation[2] = (rot[(1, 2)] + rot[(2, 1)]) / sqrt;
              local.rotation[3] = 0.25 * sqrt;
            }

            // Scaling is not supported yet. See
            // https://github.com/sebcrozet/ncollide/issues/139
            // local.scale[0] = position.scale[0];
            // local.scale[1] = position.scale[1];
            // local.scale[2] = position.scale[2];
        }
    }
}
