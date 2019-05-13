use std::cmp::Ordering;

use hibitset::BitSet;

use amethyst_core::{
    ecs::prelude::{Entities, Entity, Join, Read, ReadStorage, System, Write},
    math::{self as na, Point3, Vector3},
    Float, Transform,
};

use crate::{
    cam::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    transparent::Transparent,
};

/// Resource for controlling what entities should be rendered, and whether to draw them ordered or
/// not, which is useful for transparent surfaces.
#[derive(Default)]
pub struct Visibility {
    /// Visible entities that can be drawn in any order
    pub visible_unordered: BitSet,
    /// Visible entities that need to be drawn in the given order
    pub visible_ordered: Vec<Entity>,
}

/// Determine what entities are visible to the camera, and which are not. Will also sort transparent
/// entities back to front based on distance from camera.
///
/// Note that this should run after `GlobalTransform` has been updated for the current frame, and
/// before rendering occurs.
pub struct VisibilitySortingSystem {
    centroids: Vec<Internals>,
    transparent: Vec<Internals>,
}

#[derive(Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<Float>,
    camera_distance: Float,
    from_camera: Vector3<Float>,
}

impl VisibilitySortingSystem {
    /// Create new sorting system
    pub fn new() -> Self {
        VisibilitySortingSystem {
            centroids: Vec::default(),
            transparent: Vec::default(),
        }
    }
}

impl<'a> System<'a> for VisibilitySortingSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, Visibility>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, Transform>,
    );

    fn run(
        &mut self,
        (entities, mut visibility, hidden, hidden_prop, active, camera, transparent, transform): Self::SystemData,
    ) {
        let origin = Point3::origin();

        let camera: Option<&Transform> = active
            .entity
            .and_then(|entity| transform.get(entity))
            .or_else(|| (&camera, &transform).join().map(|cg| cg.1).next());
        let camera_backward = camera
            .map(|c| c.global_matrix().column(2).xyz())
            .unwrap_or_else(Vector3::z);
        let camera_centroid = camera
            .map(|g| g.global_matrix().transform_point(&origin))
            .unwrap_or(origin);

        self.centroids.clear();
        self.centroids.extend(
            (&*entities, &transform, !&hidden, !&hidden_prop)
                .join()
                .map(|(entity, transform, _, _)| {
                    (entity, transform.global_matrix().transform_point(&origin))
                })
                .map(|(entity, centroid)| Internals {
                    entity,
                    transparent: transparent.contains(entity),
                    centroid,
                    camera_distance: na::distance_squared(&centroid, &camera_centroid),
                    from_camera: centroid - camera_centroid,
                })
                .filter(|c| c.from_camera.dot(&camera_backward) < Float::from(0.0)), // filter entities behind the camera
        );
        self.transparent.clear();
        self.transparent
            .extend(self.centroids.iter().filter(|c| c.transparent).cloned());
        self.transparent.sort_by(|a, b| {
            b.camera_distance
                .partial_cmp(&a.camera_distance)
                .unwrap_or(Ordering::Equal)
        });
        visibility.visible_unordered.clear();
        for c in &self.centroids {
            if !c.transparent {
                visibility.visible_unordered.add(c.entity.id());
            }
        }
        visibility.visible_ordered.clear();
        visibility
            .visible_ordered
            .extend(self.transparent.iter().map(|c| c.entity));
    }
}
