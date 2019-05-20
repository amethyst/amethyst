use crate::{
    camera::{ActiveCamera, Camera},
    transparent::Transparent,
};
use amethyst_core::{
    ecs::prelude::{
        Component, DenseVecStorage, Entities, Entity, Join, Read, ReadExpect, ReadStorage, System,
        Write,
    },
    math::{self as na, convert, distance_squared, Matrix4, Point3, RealField, Vector4},
    num::One,
    Float, Hidden, HiddenPropagate, Transform,
};
use amethyst_window::ScreenDimensions;

use hibitset::BitSet;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

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
/// Note that this should run after `Transform` has been updated for the current frame, and
/// before rendering occurs.
pub struct VisibilitySortingSystem {
    centroids: Vec<Internals>,
    transparent: Vec<Internals>,
}

/// Defines a object's bounding sphere used by frustum culling.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingSphere {
    pub center: Point3<Float>,
    pub radius: Float,
}

impl Default for BoundingSphere {
    fn default() -> Self {
        Self {
            center: Point3::origin(),
            radius: na::one(),
        }
    }
}

impl BoundingSphere {
    pub fn new(center: Point3<Float>, radius: impl Into<Float>) -> Self {
        Self {
            center,
            radius: radius.into(),
        }
    }

    pub fn origin(radius: impl Into<Float>) -> Self {
        Self {
            center: Point3::origin(),
            radius: radius.into(),
        }
    }
}

impl Component for BoundingSphere {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<Float>,
    camera_distance: Float,
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
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, BoundingSphere>,
        ReadExpect<'a, ScreenDimensions>,
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
            dimensions,
        ): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("run");

        let origin = Point3::origin();
        let defcam = Camera::standard_2d(dimensions.width(), dimensions.height());
        let identity = Transform::default();

        let mut camera_join = (&camera, &transform).join();
        let (camera, camera_transform) = active
            .and_then(|a| camera_join.get(a.entity, &entities))
            .or_else(|| camera_join.next())
            .unwrap_or((&defcam, &identity));

        let camera_centroid = camera_transform.global_matrix().transform_point(&origin);
        let frustum = Frustum::new(
            convert::<_, Matrix4<Float>>(*camera.as_matrix())
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
                        sphere.map_or(na::one(), |s| s.radius)
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

#[derive(Debug)]
struct Frustum {
    planes: [Vector4<Float>; 6],
}

impl Frustum {
    fn new(matrix: Matrix4<Float>) -> Self {
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
                planes[0] * (Float::one() / planes[0].xyz().magnitude()),
                planes[1] * (Float::one() / planes[1].xyz().magnitude()),
                planes[2] * (Float::one() / planes[2].xyz().magnitude()),
                planes[3] * (Float::one() / planes[3].xyz().magnitude()),
                planes[4] * (Float::one() / planes[4].xyz().magnitude()),
                planes[5] * (Float::one() / planes[5].xyz().magnitude()),
            ],
        }
    }

    fn check_sphere(&self, center: &Point3<Float>, radius: impl Into<Float>) -> bool {
        let radius = radius.into();
        for plane in &self.planes {
            if plane.xyz().dot(&center.coords) + plane.w <= -radius {
                return false;
            }
        }
        return true;
    }
}
