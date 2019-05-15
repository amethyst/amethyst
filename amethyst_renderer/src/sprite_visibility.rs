use std::cmp::Ordering;

use hibitset::BitSet;

use amethyst_core::{
    ecs::prelude::{Entities, Entity, Join, Read, ReadStorage, System, Write},
    math::{Point3, Vector3},
    Float, Transform,
};

use crate::{
    cam::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    screen_space::ScreenSpace,
    transparent::Transparent,
};

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
    centroid: Point3<Float>,
    from_camera: Vector3<Float>,
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
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, ScreenSpace>,
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
            screen_spaces,
        ): Self::SystemData,
    ) {
        let origin = Point3::origin();

        // The camera position is used to determine culling, but the sprites are ordered based on
        // the Z coordinate
        let camera: Option<&Transform> = active
            .entity
            .and_then(|entity| transform.get(entity))
            .or_else(|| (&camera, &transform).join().map(|cg| cg.1).next());
        let camera_backward = camera
            .map(|c| c.global_matrix().column(2).xyz().into())
            .unwrap_or_else(Vector3::z);
        let camera_centroid = camera
            .map(|g| g.global_matrix().transform_point(&origin))
            .unwrap_or_else(|| origin);

        self.centroids.clear();
        self.centroids.extend(
            (
                &*entities,
                &transform,
                !&hidden,
                !&hidden_prop,
                screen_spaces.maybe(),
            )
                .join()
                .map(|(entity, transform, _, _, screen_space)| {
                    (
                        entity,
                        transform.global_matrix().transform_point(&origin),
                        screen_space,
                    )
                })
                .map(|(entity, centroid, screen_space)| {
                    (
                        Internals {
                            entity,
                            transparent: transparent.contains(entity),
                            centroid,
                            from_camera: centroid - camera_centroid,
                        },
                        screen_space,
                    )
                })
                // filter entities behind the camera
                .filter(|(c, screen_space)| {
                    c.from_camera.dot(&camera_backward) < Float::from(0.0) || screen_space.is_some()
                })
                .map(|(c, _)| c),
        );
        self.transparent.clear();
        self.transparent
            .extend(self.centroids.iter().filter(|c| c.transparent).cloned());

        // Note: Smaller Z values are placed first, so that semi-transparent sprite colors blend
        // correctly. This is opposite to the mesh visibility sorting system.
        self.transparent.sort_by(|a, b| {
            a.centroid
                .z
                .partial_cmp(&b.centroid.z)
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
