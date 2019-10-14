//! Transparency, visibility sorting and camera centroid culling for 2D Sprites.
use crate::{
    camera::{ActiveCamera, Camera},
    transparent::Transparent,
};
use amethyst_core::{
    legion::{prelude::*, Allocators, SystemDesc},
    math::{Point3, Vector3},
    Hidden, HiddenPropagate, Transform,
};
use bumpalo::{collections::Vec as BumpVec, Bump};
use derivative::Derivative;
use std::cmp::Ordering;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Resource for controlling what entities should be rendered, and whether to draw them ordered or
/// not, which is useful for transparent surfaces.
#[derive(Default, Debug)]
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
/// Note that this should run after `Transform` has been updated for the current frame, and
/// before rendering occurs.
#[derive(Debug)]
pub struct SpriteVisibilitySortingSystemState {
    allocator: Bump,
}
#[derive(Debug, Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<f32>,
    camera_distance: f32,
    from_camera: Vector3<f32>,
}

#[derive(Debug, Default)]
pub struct SpriteVisibilitySortingSystemDesc;
impl SystemDesc for SpriteVisibilitySortingSystemDesc {
    fn build(mut self, world: &mut World) -> Box<dyn Schedulable> {
        SystemBuilder::<()>::new("SpriteVisibilitySortingSystem")
            //.read_resource::<Allocators>()
            .read_resource::<ActiveCamera>()
            .write_resource::<SpriteVisibility>()
            .read_component::<Transparent>()
            .read_component::<Transform>()
            .with_query(<(Read<Camera>, Read<Transform>)>::query())
            .with_query(
                <(Read<Transform>)>::query()
                    .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
            )
            .build_disposable(
                SpriteVisibilitySortingSystemState::default(),
                |state,
                 commands,
                 world,
                 (allocators, active_camera, mut visibility),
                 (camera_query, entity_query)| {
                    let bump = Bump::default();

                    let origin = Point3::origin();

                    let camera = active_camera
                        .entity
                        .and_then(|e| world.get_component::<Transform>(e))
                        .or_else(|| {
                            camera_query
                                .iter_entities()
                                .nth(0)
                                .map(|e| world.get_component::<Transform>(e))
                        });

                    let camera_backward = camera
                        .map(|c| c.global_matrix().column(2).xyz())
                        .unwrap_or_else(Vector3::z);
                    let camera_centroid = camera
                        .map(|t| t.global_matrix().transform_point(&origin))
                        .unwrap_or_else(|| origin);

                    let mut centroids = BumpVec::from_iter_in(
                        entity_query
                            .iter_entities()
                            .map(|(e, t)| (e, t.global_matrix().transform_point(&origin)))
                            // filter entities behind the camera
                            .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                            .map(|(entity, centroid)| Internals {
                                entity,
                                transparent: world.get_component::<Transparent>(entity).is_some(),
                                centroid,
                                camera_distance: (centroid.z - camera_centroid.z).abs(),
                                from_camera: centroid - camera_centroid,
                            }),
                        &bump.bump,
                    );

                    visibility.visible_unordered.clear();
                    visibility.visible_unordered.extend(
                        self.centroids
                            .iter()
                            .filter(|c| !c.transparent)
                            .map(|c| c.entity.id()),
                    );

                    let mut transparents = BumpVec::from_iter_in(
                        centroids
                            .drain(..)
                            .filter(|c| c.transparent)
                            .sort_by(|a, b| {
                                b.camera_distance
                                    .partial_cmp(&a.camera_distance)
                                    .unwrap_or(Ordering::Equal)
                            }),
                        &bump.bump,
                    );

                    visibility.visible_ordered.clear();
                    visibility
                        .visible_ordered
                        .extend(self.transparent.iter().map(|c| c.entity));
                },
                |_| {},
            )
    }
}
/*
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
    );

    fn run(
        &mut self,
        (entities, mut visibility, hidden, hidden_prop, active, camera, transparent, transform): Self::SystemData,
    ) {



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
*/
