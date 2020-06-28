//! Helper gatherer structures for collecting information about the world.
use crate::{
    camera::{ActiveCamera, Camera},
    pod::{self, IntoPod},
    resources::AmbientColor,
};
use amethyst_core::{
    ecs::{Entities, Entity, Join, Read, ReadStorage, SystemData, World},
    math::{convert, Matrix4, Vector3},
    transform::Transform,
};
use glsl_layout::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

type Std140<T> = <T as AsStd140>::Std140;

/// Helper `CameraGatherer` for fetching appropriate matrix information from camera entities.
#[derive(Debug)]
pub struct CameraGatherer {
    /// Fetched camera world position
    pub camera_position: vec3,
    /// Fetched camera projection matrix.
    pub projview: Std140<pod::ViewArgs>,
}

impl CameraGatherer {
    /// Collect just the entity which has the current `ActiveCamera`
    pub fn gather_camera_entity(world: &World) -> Option<Entity> {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_camera (1st)");

        let (active_camera, entities, cameras, transforms) = <(
            Read<'_, ActiveCamera>,
            Entities<'_>,
            ReadStorage<'_, Camera>,
            ReadStorage<'_, Transform>,
        )>::fetch(world);

        match active_camera.entity {
            Some(entity) => {
                if transforms.contains(entity) && cameras.contains(entity) {
                    Some(entity)
                } else {
                    log::error!(
                        "The entity assigned to ActiveCamera is not a valid camera, which requires the \
                        Transform and Camera components. Falling back on the first available camera which meets these requirements");

                    (&entities, &cameras, &transforms)
                        .join()
                        .next()
                        .map(|(entity, _, _)| entity)
                }
            }
            None => (&entities, &cameras, &transforms)
                .join()
                .next()
                .map(|(entity, _, _)| entity),
        }
    }

    /// Collect `ActiveCamera` and `Camera` instances from the provided resource storage and selects
    /// the appropriate camera to use for projection, and returns the camera position and extracted
    /// projection matrix.
    ///
    /// The matrix returned is the camera's `Projection` matrix and the camera `Transform::global_view_matrix`
    pub fn gather(world: &World) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_cameras");

        let (active_camera, cameras, transforms) = <(
            Read<'_, ActiveCamera>,
            ReadStorage<'_, Camera>,
            ReadStorage<'_, Transform>,
        )>::fetch(world);

        let defcam = Camera::standard_2d(1.0, 1.0);
        let identity = Transform::default();

        let (camera, transform) = active_camera
            .entity
            .as_ref()
            .and_then(|ac| {
                cameras
                    .get(*ac)
                    .map(|camera| (camera, transforms.get(*ac).unwrap_or(&identity)))
            })
            .unwrap_or_else(|| {
                (&cameras, &transforms)
                    .join()
                    .next()
                    .unwrap_or((&defcam, &identity))
            });

        let camera_position =
            convert::<_, Vector3<f32>>(transform.global_matrix().column(3).xyz()).into_pod();

        let proj = &camera.matrix;
        let view = transform.global_view_matrix();

        let proj_view: [[f32; 4]; 4] = ((*proj) * view).into();
        let proj: [[f32; 4]; 4] = (*proj).into();
        let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(transform.global_view_matrix()).into();

        let projview = pod::ViewArgs {
            proj: proj.into(),
            view: view.into(),
            proj_view: proj_view.into(),
        }
        .std140();

        Self {
            camera_position,
            projview,
        }
    }
}

/// If an `AmbientColor` exists in the world, return it - otherwise return pure white.
#[derive(Debug)]
pub struct AmbientGatherer;
impl AmbientGatherer {
    /// If an `AmbientColor` exists in the world, return it - otherwise return pure white.
    pub fn gather(world: &World) -> vec3 {
        let ambient_color = <Option<Read<'_, AmbientColor>>>::fetch(world);
        ambient_color.map_or([0.0, 0.0, 0.0].into(), |c| {
            let (r, g, b, _) = c.0.into_components();
            [r, g, b].into()
        })
    }
}
