//! Transparency, visibility sorting and camera centroid culling for 3D Meshes.
use crate::{
    camera::Camera, legion::camera::ActiveCamera, transparent::Transparent,
    visibility::BoundingSphere, Mesh,
};
use amethyst_core::{
    legion::*,
    math::{convert, distance_squared, Matrix4, Point3, Vector4},
    Hidden, HiddenPropagate, Transform,
};

use indexmap::IndexSet;
use std::cmp::Ordering;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Resource for controlling what entities should be rendered, and whether to draw them ordered or
/// not, which is useful for transparent surfaces.
#[derive(Default, Debug)]
pub struct Visibility {
    /// Visible entities that can be drawn in any order
    pub visible_unordered: IndexSet<Entity>,
    /// Visible entities that need to be drawn in the given order
    pub visible_ordered: Vec<Entity>,
}

/// Determine what entities are visible to the camera, and which are not. Will also sort transparent
/// entities back to front based on distance from camera.
///
/// Note that this should run after `Transform` has been updated for the current frame, and
/// before rendering occurs.
#[derive(Default, Debug)]
pub struct VisibilitySortingSystemState {
    centroids: Vec<Internals>,
    transparent: Vec<Internals>,
}

#[derive(Debug, Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<f32>,
    camera_distance: f32,
}

pub fn build_visibility_sorting_system(world: &mut World) -> Box<dyn Schedulable> {
    world.resources.insert(Visibility::default());

    SystemBuilder::<()>::new("VisibilitySortingSystem")
        .read_resource::<ActiveCamera>()
        .write_resource::<Visibility>()
        .read_component::<BoundingSphere>()
        .read_component::<Transparent>()
        .with_query(<(Read<Camera>, Read<Transform>)>::query())
        .with_query(<(Read<Camera>, Read<Transform>)>::query())
        .with_query(
            <(Read<Transform>)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
        )
        .build_disposable(
            VisibilitySortingSystemState::default(),
            |state,
             commands,
             world,
             (active_camera, visibility),
             (camera_query1, camera_query2, entity_query)| {
                #[cfg(feature = "profiler")]
                profile_scope!("visibility_sorting_system");

                visibility.visible_unordered.clear();
                visibility.visible_ordered.clear();
                state.transparent.clear();
                state.centroids.clear();

                let origin = Point3::origin();

                let (camera, camera_transform) = active_camera.entity.map_or_else(
                    || {
                        camera_query1
                            .iter_entities()
                            .nth(0)
                            .map(|args| args.1)
                            .expect("No cameras are currently added to the world!")
                    },
                    |e| {
                        camera_query2
                            .iter_entities()
                            .find(|(camera_entity, (_, _))| *camera_entity == e)
                            .map(|args| args.1)
                            .expect("Invalid entity set as ActiveCamera!")
                    },
                );

                let camera_centroid = camera_transform.global_matrix().transform_point(&origin);
                let frustum = Frustum::new(
                    convert::<_, Matrix4<f32>>(*camera.as_matrix())
                        * camera_transform.global_matrix().try_inverse().unwrap(),
                );

                state.centroids.extend(
                    entity_query
                        .iter_entities()
                        .map(|(entity, transform)| {
                            let sphere = world.get_component::<BoundingSphere>(entity);

                            let pos = sphere.clone().map_or(origin, |s| s.center);
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
                            transparent: world.get_component::<Transparent>(entity).is_some(),
                            centroid,
                            camera_distance: distance_squared(&centroid, &camera_centroid),
                        }),
                );

                state
                    .transparent
                    .extend(state.centroids.iter().filter(|c| c.transparent).cloned());

                state.transparent.sort_by(|a, b| {
                    b.camera_distance
                        .partial_cmp(&a.camera_distance)
                        .unwrap_or(Ordering::Equal)
                });

                visibility.visible_unordered.extend(
                    state
                        .centroids
                        .iter()
                        .filter(|c| !c.transparent)
                        .map(|c| c.entity),
                );

                visibility
                    .visible_ordered
                    .extend(state.transparent.iter().map(|c| c.entity));
            },
            |_, _| {},
        )
}

/*
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
            convert::<_, Matrix4<f32>>(*camera.as_matrix())
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
*/

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
