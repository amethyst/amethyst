use std::cmp::Ordering;

use amethyst_core::GlobalTransform;
use amethyst_core::cgmath::{EuclideanSpace, InnerSpace, MetricSpace, Point3, Transform, Vector3};
use specs::{Component, Entities, Entity, Fetch, FetchMut, Join, NullStorage, ReadStorage, System};

use cam::{ActiveCamera, Camera};

/// Transparent mesh component
#[derive(Clone, Debug, Default)]
pub struct Transparent;

impl Component for Transparent {
    type Storage = NullStorage<Self>;
}

/// Transparent mesh entities sorted back to front
#[derive(Clone, Debug, Default)]
pub struct TransparentBackToFront {
    /// Entities
    pub entities: Vec<Entity>,
}

/// Sort transparent entities back to front using the active camera.
///
/// Note that this should run after `GlobalTransform` has been updated for the current frame, and
/// before rendering occurs.
pub struct TransparentSortingSystem {
    centroids: Vec<(Entity, Point3<f32>, f32, Vector3<f32>)>,
}

impl TransparentSortingSystem {
    /// Create new sorting system
    pub fn new() -> Self {
        TransparentSortingSystem {
            centroids: Vec::default(),
        }
    }
}

impl<'a> System<'a> for TransparentSortingSystem {
    type SystemData = (
        Entities<'a>,
        FetchMut<'a, TransparentBackToFront>,
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, GlobalTransform>,
    );

    fn run(
        &mut self,
        (entities, mut back_to_front, active, camera, transparent, global): Self::SystemData,
    ) {
        let origin = Point3::origin();

        let camera: Option<&GlobalTransform> = active
            .and_then(|a| global.get(a.entity))
            .or_else(|| (&camera, &global).join().map(|cg| cg.1).next());
        let camera_forward = camera
            .map(|c| c.0.z.truncate())
            .unwrap_or(Vector3::unit_z());
        let camera_centroid = camera
            .map(|g| g.0.transform_point(origin))
            .unwrap_or(origin.clone());

        self.centroids = (&*entities, &transparent, &global)
            .join()
            .map(|(entity, _, global)| (entity, global.0.transform_point(origin)))
            .map(|(entity, centroid)| {
                (
                    entity,
                    centroid,
                    centroid.distance2(camera_centroid),
                    centroid - camera_centroid,
                )
            })
            .filter(|c| c.3.dot(camera_forward) > 0.) // filter entities behind the camera
            .collect();
        self.centroids
            .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));
        back_to_front.entities = self.centroids.iter().map(|c| c.0).collect();
    }
}
