use amethyst_core::cgmath::prelude::*;
use amethyst_core::cgmath::Point3;
use amethyst_core::specs::prelude::*;
use amethyst_core::specs::Entity;
use amethyst_core::transform::GlobalTransform;

use collision::prelude::*;
use collision::primitive::Primitive3;
use collision::{Plane, Ray3};

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

pub enum AB<A, B> {
    A(A),
    B(B),
}

/// Component which indicates an entity should be checked for intersections against the `MouseRay`.
pub struct Pickable {
    /// The bounding-hull of the entity in pre-transformation coordinates (eg. unscaled mesh size).
    pub bounds: AB<Primitive3<f32>, Plane<f32>>,
}

impl Component for Pickable {
    type Storage = DenseVecStorage<Self>;
}

pub struct PickSys;
/// System which intersects the `MouseRay` against every `Pickable` entity.

impl<'s> System<'s> for PickSys {
    type SystemData = (
        Entities<'s>,
        Read<'s, MouseRay>,
        ReadStorage<'s, GlobalTransform>,
        ReadStorage<'s, Pickable>,
        Write<'s, Picked>,
    );
    fn run(&mut self, (entity, mouseray, transform, pickable, mut picked): Self::SystemData) {
        let ray = mouseray.ray();
        // Search for the nearest intersecting entity
        let mut nearest = None;
        for (entity, transform, pickable) in (&*entity, &transform, &pickable).join() {
            match pickable.bounds {
                // FIXME: these two branches are identical; try to use a generic in Pickable<T>
                AB::A(ref bounds) => update_nearest(&mut nearest, &ray, entity, transform, bounds),
                AB::B(ref bounds) => update_nearest(&mut nearest, &ray, entity, transform, bounds),
            };
        }
        // Write the nearest intersected entity to the picked resource
        picked.entity_intersection = nearest.map(|(_, point, entity)| (entity, point));
    }
}

/// - Check intersection of ray with obj_hull transformed by obj_trans.
/// - If there's an intersection and it's nearer than the previously stored one,
///   Then store the new intersection point, distance, and obj_entity.
fn update_nearest<T>(
    nearest: &mut Option<(f32, Point3<f32>, Entity)>,
    ray: &Ray3<f32>,
    obj_entity: Entity,
    obj_trans: &GlobalTransform,
    obj_hull: &T,
) where
    T: ContinuousTransformed<Ray3<f32>, Point = Point3<f32>, Result = Point3<f32>>,
{
    obj_hull
        .intersection_transformed(&ray, &obj_trans.0)
        .map(|point| {
            let dist2 = ray.origin.distance2(point);
            if nearest.map_or(true, |(neardist2, _, _)| dist2 < neardist2) {
                *nearest = Some((dist2, point, obj_entity))
            }
        });
}
