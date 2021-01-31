//! Transparency, visibility sorting and camera centroid culling for 2D Sprites.
use std::cmp::Ordering;

use amethyst_core::{
    ecs::*,
    math::{Point3, Vector3},
    transform::Transform,
    Hidden, HiddenPropagate,
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    camera::{ActiveCamera, Camera},
    sprite::SpriteRender,
    transparent::Transparent,
};

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

/// Determines what entities to be drawn. Will also sort transparent entities back to front based on
/// position on the Z axis.
///
/// The sprite render pass should draw all sprites without semi-transparent pixels, then draw the
/// sprites with semi-transparent pixels from far to near.
///
/// Note that this should run after `Transform` has been updated for the current frame, and
/// before rendering occurs.
#[derive(Debug)]
pub struct SpriteVisibilitySortingSystem;

impl System for SpriteVisibilitySortingSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        let mut transparent_centroids: Vec<Internals> = Vec::default();

        Box::new(
            SystemBuilder::<()>::new("SpriteVisibilitySortingSystem")
                .read_resource::<ActiveCamera>()
                .write_resource::<SpriteVisibility>()
                .with_query(<(&Camera, &Transform)>::query())
                .with_query(<(Entity, &Camera, &Transform)>::query())
                .with_query(
                    <(Entity, &Transform, &SpriteRender, &Transparent)>::query()
                        .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
                )
                .with_query(<(Entity, &Transform, &SpriteRender)>::query().filter(
                    !component::<Transparent>()
                        & !component::<Hidden>()
                        & !component::<HiddenPropagate>(),
                ))
                .build(
                    move |commands,
                          world,
                          (active_camera, visibility),
                          (
                        camera_query1,
                        camera_query2,
                        transparent_query,
                        non_transparent_query,
                    )| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("sprite_visibility_system");

                        transparent_centroids.clear();
                        visibility.visible_ordered.clear();
                        visibility.visible_unordered.clear();

                        let origin = Point3::origin();

                        let (camera, camera_transform) = match active_camera.entity.map_or_else(
                            || camera_query1.iter(world).next(),
                            |e| {
                                camera_query2
                                    .iter(world)
                                    .find(|(camera_entity, _, _)| **camera_entity == e)
                                    .map(|(_entity, camera, camera_transform)| {
                                        (camera, camera_transform)
                                    })
                            },
                        ) {
                            Some(r) => r,
                            None => return,
                        };

                        let camera_backward = camera_transform.global_matrix().column(2).xyz();
                        let camera_centroid =
                            camera_transform.global_matrix().transform_point(&origin);

                        transparent_centroids.extend(
                            transparent_query
                                .iter(world)
                                .map(|(e, t, _, _)| {
                                    (*e, t.global_matrix().transform_point(&origin))
                                })
                                // filter entities behind the camera
                                .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                                .map(|(entity, centroid)| {
                                    Internals {
                                        entity,
                                        centroid,
                                        camera_distance: (centroid.z - camera_centroid.z).abs(),
                                        from_camera: centroid - camera_centroid,
                                    }
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
                                .iter(world)
                                .map(|(e, t, _)| (e, t.global_matrix().transform_point(&origin)))
                                // filter entities behind the camera
                                .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                                .map(|(entity, _)| entity),
                        );
                    },
                ),
        )
    }
}
