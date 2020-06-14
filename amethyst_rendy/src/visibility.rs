//! Transparency, visibility sorting and camera centroid culling for 3D Meshes.
use crate::{
    camera::{ActiveCamera, Camera},
    transparent::Transparent,
};
use amethyst_core::{
    ecs::{
        hibitset::BitSet,
        prelude::{
            Component, DenseVecStorage, Entities, Entity, Join, Read, ReadStorage, System, Write,
        },
    },
    math::{convert, distance_squared, Matrix4, Point3, Vector4},
    Hidden, HiddenPropagate, Transform,
};

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Resource for controlling what entities should be rendered, and whether to draw them ordered or
/// not, which is useful for transparent surfaces.
#[derive(Default, Debug)]
pub struct Visibility {
    /// Visible entities that can be drawn in any order
    pub visible_unordered: BitSet,
    /// Visible entities that need to be drawn in the given order
    pub visible_ordered: Vec<Entity>,
}

/// Determine what entities are visible to the camera, and which are not. Will also sort transparent
/// entities back to front based on distance from camera.
///
/// Note that this should run after `Transform` has been updated for the current frame, and
/// before rendering occurs.
#[derive(Default, Debug)]
pub struct VisibilitySortingSystem {
    centroids: Vec<Internals>,
    transparent: Vec<Internals>,
}

/// Defines a object's bounding sphere used by frustum culling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingSphere {
    /// Center of the bounding sphere
    pub center: Point3<f32>,
    /// Radius of the bounding sphere.
    pub radius: f32,
}

impl Default for BoundingSphere {
    fn default() -> Self {
        Self {
            center: Point3::origin(),
            radius: 1.0,
        }
    }
}

impl BoundingSphere {
    /// Create a new `BoundingSphere` with the supplied radius and center.
    pub fn new(center: Point3<f32>, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Returns the center of the sphere.
    pub fn origin(radius: f32) -> Self {
        Self {
            center: Point3::origin(),
            radius,
        }
    }
}

impl Component for BoundingSphere {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<f32>,
    camera_distance: f32,
}

impl VisibilitySortingSystem {
    /// Create new sorting system
    pub fn new() -> Self {
        Self::default()
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
        ReadStorage<'a, BoundingSphere>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut visibility,
            hidden,
            hidden_prop,
            active,
            camera,
            transparent,
            transform,
            bound,
        ): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("visibility_sorting_system");

        let origin = Point3::origin();
        let defcam = Camera::standard_2d(1.0, 1.0);
        let identity = Transform::default();

        let mut camera_join = (&camera, &transform).join();
        let (camera, camera_transform) = active
            .entity
            .and_then(|a| camera_join.get(a, &entities))
            .or_else(|| camera_join.next())
            .unwrap_or((&defcam, &identity));

        let camera_centroid = camera_transform.global_matrix().transform_point(&origin);
        let frustum = Frustum::new(
            convert::<_, Matrix4<f32>>(camera.matrix)
                * camera_transform.global_matrix().try_inverse().unwrap(),
        );

        self.centroids.clear();
        self.centroids.extend(
            (
                &*entities,
                &transform,
                bound.maybe(),
                !&hidden,
                !&hidden_prop,
            )
                .join()
                .map(|(entity, transform, sphere, _, _)| {
                    let pos = sphere.map_or(&origin, |s| &s.center);
                    let matrix = transform.global_matrix();
                    (
                        entity,
                        matrix.transform_point(&pos),
                        sphere.map_or(1.0, |s| s.radius)
                            * matrix[(0, 0)].max(matrix[(1, 1)]).max(matrix[(2, 2)]),
                    )
                })
                .filter(|(_, centroid, radius)| frustum.check_sphere(centroid, *radius))
                .map(|(entity, centroid, _)| Internals {
                    entity,
                    transparent: transparent.contains(entity),
                    centroid,
                    camera_distance: distance_squared(&centroid, &camera_centroid),
                }),
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
        visibility.visible_unordered.extend(
            self.centroids
                .iter()
                .filter(|c| !c.transparent)
                .map(|c| c.entity.id()),
        );

        visibility.visible_ordered.clear();
        visibility
            .visible_ordered
            .extend(self.transparent.iter().map(|c| c.entity));
    }
}

/// Simple view Frustum implementation
#[derive(Debug)]
pub struct Frustum {
    /// The planes of the frustum
    pub planes: [Vector4<f32>; 6],
}

impl Frustum {
    /// Create a new simple frustum from the provided matrix.
    pub fn new(matrix: Matrix4<f32>) -> Self {
        let planes = [
            (matrix.row(3) + matrix.row(0)).transpose(),
            (matrix.row(3) - matrix.row(0)).transpose(),
            (matrix.row(3) - matrix.row(1)).transpose(),
            (matrix.row(3) + matrix.row(1)).transpose(),
            (matrix.row(3) + matrix.row(2)).transpose(),
            (matrix.row(3) - matrix.row(2)).transpose(),
        ];
        Self {
            planes: [
                planes[0] * (1.0 / planes[0].xyz().magnitude()),
                planes[1] * (1.0 / planes[1].xyz().magnitude()),
                planes[2] * (1.0 / planes[2].xyz().magnitude()),
                planes[3] * (1.0 / planes[3].xyz().magnitude()),
                planes[4] * (1.0 / planes[4].xyz().magnitude()),
                planes[5] * (1.0 / planes[5].xyz().magnitude()),
            ],
        }
    }

    /// Check if the given sphere is within the Frustum
    pub fn check_sphere(&self, center: &Point3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.xyz().dot(&center.coords) + plane.w <= -radius {
                return false;
            }
        }
        true
    }
}
