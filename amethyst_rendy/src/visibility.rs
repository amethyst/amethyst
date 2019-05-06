use crate::{
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    transparent::Transparent,
};
use amethyst_core::{
    alga::general::SubsetOf,
    ecs::prelude::{
        Component, DenseVecStorage, Entities, Entity, Join, Read, ReadStorage, System, Write,
    },
    math::{distance_squared, try_convert, Matrix4, Point3, RealField, Vector4},
    Transform,
};
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
pub struct VisibilitySortingSystem<N: RealField> {
    centroids: Vec<Internals<N>>,
    transparent: Vec<Internals<N>>,
}

/// Defines a object's bounding sphere used by frustum culling.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingSphere<N: RealField> {
    pub center: Point3<N>,
    pub radius: N,
}

impl<N: RealField> Default for BoundingSphere<N> {
    fn default() -> Self {
        Self {
            center: Point3::origin(),
            radius: N::one(),
        }
    }
}

impl<N: RealField> BoundingSphere<N> {
    pub fn new(center: Point3<N>, radius: N) -> Self {
        Self { center, radius }
    }

    pub fn origin(radius: N) -> Self {
        Self {
            center: Point3::origin(),
            radius,
        }
    }
}

impl<N: RealField> Component for BoundingSphere<N> {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone)]
struct Internals<N: RealField> {
    entity: Entity,
    transparent: bool,
    centroid: Point3<N>,
    camera_distance: N,
}

impl<N: RealField> VisibilitySortingSystem<N> {
    /// Create new sorting system
    pub fn new() -> Self {
        VisibilitySortingSystem {
            centroids: Vec::default(),
            transparent: Vec::default(),
        }
    }
}

impl<'a, N: RealField + SubsetOf<f32>> System<'a> for VisibilitySortingSystem<N> {
    type SystemData = (
        Entities<'a>,
        Write<'a, Visibility>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, Transform<N>>,
        ReadStorage<'a, BoundingSphere<N>>,
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
        profile_scope!("run");

        let origin = Point3::origin();
        let defcam = Camera::standard_2d();
        let identity = Transform::default();

        let mut camera_join = (&camera, &transform).join();
        let (camera, camera_transform) = active
            .and_then(|a| camera_join.get(a.entity, &entities))
            .or_else(|| camera_join.next())
            .unwrap_or((&defcam, &identity));

        let camera_centroid = camera_transform.global_matrix().transform_point(&origin);
        let frustum = Frustum::<N>::new(
            try_convert::<Matrix4<f32>, Matrix4<N>>(camera.proj).unwrap()
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
                        sphere.map_or(N::one(), |s| s.radius)
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
struct Frustum<N: RealField> {
    planes: [Vector4<N>; 6],
}

impl<N: RealField> Frustum<N> {
    fn new(matrix: Matrix4<N>) -> Self {
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
                planes[0] * (N::one() / planes[0].xyz().magnitude()),
                planes[1] * (N::one() / planes[1].xyz().magnitude()),
                planes[2] * (N::one() / planes[2].xyz().magnitude()),
                planes[3] * (N::one() / planes[3].xyz().magnitude()),
                planes[4] * (N::one() / planes[4].xyz().magnitude()),
                planes[5] * (N::one() / planes[5].xyz().magnitude()),
            ],
        }
    }

    fn check_sphere(&self, center: &Point3<N>, radius: N) -> bool {
        for plane in &self.planes {
            if plane.xyz().dot(&center.coords) + plane.w <= -radius {
                return false;
            }
        }
        return true;
    }
}
