use crate::{
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    transparent::Transparent,
};
use amethyst_core::{
    ecs::prelude::{Entities, Entity, Join, Read, ReadStorage, System, Write},
    math::{Point3, Vector3},
    GlobalTransform,
};
use hibitset::BitSet;
use std::cmp::Ordering;

/// Resource for controlling what entities should be rendered, and whether to draw them ordered or
/// not, which is useful for transparent surfaces.
#[derive(Default)]
pub struct SpriteVisibility {
    /// Visible entities that can be drawn in any order
    pub visible_unordered: BitSet,
    /// Visible entities that need to be drawn in the given order
    pub visible_ordered: Vec<Entity>,
}

/// Determines what entities to be drawn. Will also sort transparent entities back to front based on
/// position on the Z axis.
///
/// The sprite render pass should draw all sprites without semi-transparent pixels, then draw the
/// sprites with semi-transparent pixels from far to near.
///
/// Note that this should run after `GlobalTransform` has been updated for the current frame, and
/// before rendering occurs.
#[derive(Default)]
pub struct SpriteVisibilitySortingSystem {
    centroids: Vec<Internals>,
    transparent: Vec<Internals>,
}

#[derive(Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<f32>,
    camera_distance: f32,
    from_camera: Vector3<f32>,
}

impl SpriteVisibilitySortingSystem {
    /// Returns a new sprite visibility sorting system
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for SpriteVisibilitySortingSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, SpriteVisibility>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, GlobalTransform>,
    );

    fn run(
        &mut self,
        (entities, mut visibility, hidden, hidden_prop, active, camera, transparent, global): Self::SystemData,
    ) {
        let origin = Point3::origin();

        // The camera position is used to determine culling, but the sprites are ordered based on
        // the Z coordinate
        let camera: Option<&GlobalTransform> = active
            .and_then(|a| global.get(a.entity))
            .or_else(|| (&camera, &global).join().map(|cg| cg.1).next());
        let camera_backward = camera
            .map(|c| c.0.column(2).xyz().into())
            .unwrap_or_else(Vector3::z);
        let camera_centroid = camera
            .map(|g| g.0.transform_point(&origin))
            .unwrap_or_else(|| origin);

        self.centroids.clear();
        self.centroids.extend(
            (&*entities, &global, !&hidden, !&hidden_prop)
                .join()
                .map(|(entity, global, _, _)| (entity, global.0.transform_point(&origin)))
                // filter entities behind the camera
                .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.)
                .map(|(entity, centroid)| Internals {
                    entity,
                    transparent: transparent.contains(entity),
                    centroid,
                    camera_distance: (centroid.z - camera_centroid.z).abs(),
                    from_camera: centroid - camera_centroid,
                }),
        );

        visibility.visible_unordered.clear();
        visibility.visible_unordered.extend(
            self.centroids
                .iter()
                .filter(|c| !c.transparent)
                .map(|c| c.entity.id()),
        );

        self.transparent.clear();
        self.transparent
            .extend(self.centroids.drain(..).filter(|c| c.transparent));

        // Note: Smaller Z values are placed first, so that semi-transparent sprite colors blend
        // correctly.
        self.transparent.sort_by(|a, b| {
            b.camera_distance
                .partial_cmp(&a.camera_distance)
                .unwrap_or(Ordering::Equal)
        });

        visibility.visible_ordered.clear();
        visibility
            .visible_ordered
            .extend(self.transparent.iter().map(|c| c.entity));
    }
}
