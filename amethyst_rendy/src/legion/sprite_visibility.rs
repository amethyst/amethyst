//! Transparency, visibility sorting and camera centroid culling for 2D Sprites.
use crate::{
    camera::Camera, legion::camera::ActiveCamera, sprite::SpriteRender, transparent::Transparent,
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

#[derive(Debug, Clone)]
struct Internals {
    entity: Entity,
    centroid: Point3<f32>,
    camera_distance: f32,
    from_camera: Vector3<f32>,
}

pub fn build_sprite_visibility_sorting_system(world: &mut World) -> Box<dyn Schedulable> {
    world.resources.insert(SpriteVisibility::default());

    let mut transparent_centroids: Vec<Internals> = Vec::default();

    SystemBuilder::<()>::new("SpriteVisibilitySortingSystem")
        .read_resource::<ActiveCamera>()
        .write_resource::<SpriteVisibility>()
        .with_query(<(Read<Camera>, Read<Transform>)>::query())
        .with_query(<(Read<Camera>, Read<Transform>)>::query())
        .with_query(
            <(Read<Transform>, Read<SpriteRender>, Read<Transparent>)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
        )
        .with_query(<(Read<Transform>, Read<SpriteRender>)>::query().filter(
            !component::<Transparent>() & !component::<Hidden>() & !component::<HiddenPropagate>(),
        ))
        .build(
            move |commands,
                  world,
                  (active_camera, visibility),
                  (camera_query1, camera_query2, transparent_query, non_transparent_query)| {
                transparent_centroids.clear();
                visibility.visible_ordered.clear();
                visibility.visible_unordered.clear();

                let origin = Point3::origin();

                let camera_transform = active_camera.entity.map_or_else(
                    || {
                        camera_query1
                            .iter_entities()
                            .nth(0)
                            .map(|(e, (camera, transform))| transform)
                            .expect("No cameras are currently added to the world!")
                    },
                    |e| {
                        camera_query2
                            .iter_entities()
                            .find(|(camera_entity, (_, transform))| *camera_entity == e)
                            .map(|(camera_entity, (_, transform))| transform)
                            .expect("Invalid entity set as ActiveCamera!")
                    },
                );

                let camera_backward = camera_transform.global_matrix().column(2).xyz();
                let camera_centroid = camera_transform.global_matrix().transform_point(&origin);

                transparent_centroids.extend(
                    transparent_query
                        .iter_entities()
                        .map(|(e, (t, _, _))| (e, t.global_matrix().transform_point(&origin)))
                        // filter entities behind the camera
                        .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                        .map(|(entity, centroid)| Internals {
                            entity,
                            centroid,
                            camera_distance: (centroid.z - camera_centroid.z).abs(),
                            from_camera: centroid - camera_centroid,
                        }),
                );

                transparent_centroids.sort_by(|a, b| {
                    b.camera_distance
                        .partial_cmp(&a.camera_distance)
                        .unwrap_or(Ordering::Equal)
                });

                visibility
                    .visible_ordered
                    .extend(transparent_centroids.iter().map(|c| c.entity));

                visibility.visible_unordered.extend(
                    non_transparent_query
                        .iter_entities()
                        .map(|(e, (t, _))| (e, t.global_matrix().transform_point(&origin)))
                        // filter entities behind the camera
                        .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                        .map(|(entity, _)| entity),
                );
            },
        )
}
