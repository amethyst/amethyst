use amethyst_core::cgmath::prelude::*;
use amethyst_core::cgmath::Point3;
use amethyst_core::specs::prelude::*;
use amethyst_core::specs::Entity;
use amethyst_core::transform::GlobalTransform;

use collision::prelude::*;
use collision::primitive::Primitive3;
use collision::{Plane, Ray3};

use super::mouseray::MouseRay;

/// Resource indicating the entity which the mouseray intersects
pub struct Picked {
    pub entity_intersection: Option<(Entity, Point3<f32>)>,
}

impl Picked {
    fn is_entity(&self, other: Entity) -> bool {
        self.entity_intersection
            .map_or(false, |(entity, _)| entity == other)
    }
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

/// Component with a bounding box which can be compared with the mouseray
pub struct Pickable {
    // TODO: try to remove this in favor of a generic .. will require a trait bound on
    // ContinuousTransform in PickSys, and lots of lifetime bs
    pub bounds: AB<Primitive3<f32>, Plane<f32>>,
}

impl Component for Pickable {
    type Storage = DenseVecStorage<Self>;
}

/// System to write the Picked resource by comparing bounding boxes to the mouseray
pub struct PickSys;

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

/// - check intersection of ray with obj_hull transformed by obj_trans
/// - if there's an intersection and it's nearer than the previously stored one
/// - then store the new intersection point, distance, and obj_entity
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
