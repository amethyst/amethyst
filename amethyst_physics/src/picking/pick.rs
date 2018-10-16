use std::marker::PhantomData;

use amethyst_core::cgmath::prelude::*;
use amethyst_core::cgmath::Point3;
use amethyst_core::specs::prelude::*;
use amethyst_core::specs::Entity;
use amethyst_core::transform::GlobalTransform;

use collision::{ContinuousTransformed, Ray3};

use super::mouseray::MouseRay;

/// Resource indicating the entity which the `MouseRay` intersects and the point of intersection.
pub struct Picked {
    /// Contains the entity and intersection point only when there is an active intersection.
    /// Otherwise empty.
    pub entity_intersection: Option<(Entity, Point3<f32>)>,
}

impl Default for Picked {
    fn default() -> Self {
        Picked {
            entity_intersection: None,
        }
    }
}

/// Component which indicates an entity should be checked for intersections against the `MouseRay`.
pub struct Pickable<T> {
    /// The bounding-hull of the entity in pre-transformation coordinates (eg. unscaled mesh size).
    pub bounds: T,
}

impl<T> Component for Pickable<T>
where
    T: 'static + Send + Sync,
{
    type Storage = DenseVecStorage<Self>;
}

/// System which intersects the `MouseRay` against every `Pickable` entity.
pub struct PickSys<T> {
    _marker: PhantomData<T>,
}

impl<T> PickSys<T> {
    /// Initialize a new `PickSys`.
    pub fn new() -> Self {
        PickSys {
            _marker: PhantomData,
        }
    }
}

impl<'s, T> System<'s> for PickSys<T>
where
    T: 'static
        + Send
        + Sync
        + ContinuousTransformed<Ray3<f32>, Point = Point3<f32>, Result = Point3<f32>>,
{
    type SystemData = (
        Entities<'s>,
        Read<'s, MouseRay>,
        ReadStorage<'s, GlobalTransform>,
        ReadStorage<'s, Pickable<T>>,
        Write<'s, Picked>,
    );
    fn run(&mut self, (entity, mouseray, transform, pickable, mut picked): Self::SystemData) {
        let MouseRay(ray) = *mouseray;
        // Search for the nearest intersecting entity
        let mut nearest = None;
        for (entity, transform, pickable) in (&*entity, &transform, &pickable).join() {
            pickable
                .bounds
                .intersection_transformed(&ray, &transform.0)
                .map(|point| {
                    let dist2 = ray.origin.distance2(point);
                    if nearest.map_or(true, |(neardist2, _, _)| dist2 < neardist2) {
                        nearest = Some((dist2, point, entity))
                    }
                });
        }
        // Write the nearest intersected entity to the picked resource
        picked.entity_intersection = nearest.map(|(_, point, entity)| (entity, point));
    }
}
