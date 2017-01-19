extern crate ncollide;
extern crate nphysics3d;

use std::ops::{Deref, DerefMut};

use ecs::{Component, VecStorage};


// Reëxport `ncollide` shapes
pub use self::ncollide::shape::{Ball, Plane, Cuboid, Capsule, Cone, Cylinder, ConvexHull,
                                MinkowskiSum, AnnotatedMinkowskiSum, AnnotatedPoint, Reflection,
                                Compound, BaseMesh, BaseMeshElement, TriMesh, TriMesh3, Polyline,
                                Segment, Triangle, Torus, CompositeShape, SupportMap};

// Also reëxport some `nphysics3d` objects
pub use self::nphysics3d::object::{RigidBody, RigidBodyHandle, Sensor, SensorHandle};


/// Wrapper around `nphysics`'s `RigidBodyHandle` that impls some convenience
/// traits, along with `Send`/`Sync`, so that it can be processed by the ECS
/// system. We take care of ensuring that there are no data races inside the
/// ECS system, but the compiler can't infer this, so we have to manually tell
/// it that it's ok.
pub struct PhysicsComponent(pub RigidBodyHandle<f32>);

unsafe impl Sync for PhysicsComponent { }
unsafe impl Send for PhysicsComponent { }

impl Component for PhysicsComponent {
    type Storage = VecStorage<PhysicsComponent>;
}

impl Deref for PhysicsComponent {
    type Target = RigidBodyHandle<f32>;
    fn deref(&self) -> &RigidBodyHandle<f32> {
        &self.0
    }
}

impl DerefMut for PhysicsComponent {
    fn deref_mut(&mut self) -> &mut RigidBodyHandle<f32> {
        &mut self.0
    }
}
