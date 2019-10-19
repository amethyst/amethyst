//! Transparency, visibility sorting and camera centroid culling for 2D Sprites.
use crate::{
    camera::{Camera, LegionActiveCamera},
    sprite::SpriteRender,
    transparent::Transparent,
};
use amethyst_core::{
    legion::*,
    math::{Point3, Vector3},
    Hidden, HiddenPropagate, Transform,
};
use derivative::Derivative;
use indexmap::IndexSet;

use std::cmp::Ordering;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Resource for controlling what entities should be rendered, and whether to draw them ordered or
/// not, which is useful for transparent surfaces.
#[derive(Default, Debug)]
pub struct SpriteVisibility {
    /// Visible entities that can be drawn in any order
    pub visible_unordered: Vec<Entity>,
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
#[derive(Default, Debug)]
pub struct SpriteVisibilitySortingSystemState {
    transparent_centroids: Vec<Internals>,
    nontransparent_centroids: Vec<Internals>,
}
#[derive(Debug, Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<f32>,
    camera_distance: f32,
    from_camera: Vector3<f32>,
}

pub fn build_sprite_visibility_sorting_system(world: &mut World) -> Box<dyn Schedulable> {
    world.resources.insert(SpriteVisibility::default());

    SystemBuilder::<()>::new("SpriteVisibilitySortingSystem")
        //.read_resource::<Allocators>()
        //.read_resource::<LegionActiveCamera>()
        .write_resource::<SpriteVisibility>()
        .read_component::<Transparent>()
        .read_component::<Transform>()
        .with_query(<(Read<Camera>, Read<Transform>)>::query())
        .with_query(<(Read<Transform>, Read<SpriteRender>)>::query().filter(
            !component::<Transparent>() & !component::<Hidden>() & !component::<HiddenPropagate>(),
        ))
        .with_query(
            <(Read<Transform>, Read<SpriteRender>, Read<Transparent>)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
        )
        .build_disposable(
            SpriteVisibilitySortingSystemState::default(),
            |state,
             commands,
             world,
             (visibility),
             (camera_query, transparent_query, non_transparent_query)| {
                state.transparent_centroids.clear();
                state.nontransparent_centroids.clear();

                let origin = Point3::origin();

                /* TODO: no legion active camera for now, LegionActiveCamera not used
                let camera = active_camera
                    .entity
                    .and_then(|e| *world.get_component::<Transform>(e))
                    .or_else(|| {
                        camera_query
                            .iter_entities()
                            .nth(0)
                            .map(|(e, _)| world.get_component::<Transform>(e))
                    });
                */

                let camera_transform = camera_query
                    .iter_entities()
                    .nth(0)
                    .map(|(e, (camera, transform))| transform);

                let camera_backward = camera_transform
                    .as_ref()
                    .map(|c| c.global_matrix().column(2).xyz())
                    .unwrap_or_else(Vector3::z);
                let camera_centroid = camera_transform
                    .as_ref()
                    .map(|t| t.global_matrix().transform_point(&origin))
                    .unwrap_or_else(|| origin);

                state.transparent_centroids.extend(
                    transparent_query
                        .iter_entities()
                        .map(|(e, (t, _))| (e, t.global_matrix().transform_point(&origin)))
                        // filter entities behind the camera
                        .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                        .map(|(entity, centroid)| Internals {
                            entity,
                            transparent: world.get_component::<Transparent>(entity).is_some(),
                            centroid,
                            camera_distance: (centroid.z - camera_centroid.z).abs(),
                            from_camera: centroid - camera_centroid,
                        }),
                );

                state.transparent_centroids.sort_by(|a, b| {
                    b.camera_distance
                        .partial_cmp(&a.camera_distance)
                        .unwrap_or(Ordering::Equal)
                });

                state.nontransparent_centroids.extend(
                    non_transparent_query
                        .iter_entities()
                        .map(|(e, (t, _, _))| (e, t.global_matrix().transform_point(&origin)))
                        // filter entities behind the camera
                        .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                        .map(|(entity, centroid)| Internals {
                            entity,
                            transparent: world.get_component::<Transparent>(entity).is_some(),
                            centroid,
                            camera_distance: (centroid.z - camera_centroid.z).abs(),
                            from_camera: centroid - camera_centroid,
                        }),
                );

                visibility
                    .visible_unordered
                    .extend(state.nontransparent_centroids.iter().map(|c| c.entity));
                visibility
                    .visible_ordered
                    .extend(state.transparent_centroids.iter().map(|c| c.entity))
            },
            |_, _| {},
        )
}
