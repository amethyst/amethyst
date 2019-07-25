//! Helper gatherer structures for collecting information about the world.
use crate::{
    camera::{ActiveCamera, Camera},
    pod::{self, IntoPod},
    resources::AmbientColor,
};
use amethyst_core::{
    ecs::{Join, Read, ReadStorage, Resources, SystemData},
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
    /// Collect `ActiveCamera` and `Camera` instances from the provided resource storage and selects
    /// the appropriate camera to use for projection, and returns the camera position and extracted
    /// projection matrix.
    ///
    /// The matrix returned is the camera's `Projection` matrix and the camera `Transform::global_view_matrix`
    pub fn gather(res: &Resources) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_cameras");

        let (active_camera, cameras, transforms) = <(
            Read<'_, ActiveCamera>,
            ReadStorage<'_, Camera>,
            ReadStorage<'_, Transform>,
        )>::fetch(res);

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

        let proj: [[f32; 4]; 4] = (*camera.as_matrix()).into();
        let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(transform.global_view_matrix()).into();

        let projview = pod::ViewArgs {
            proj: proj.into(),
            view: view.into(),
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
    pub fn gather(res: &Resources) -> vec3 {
        let ambient_color = <Option<Read<'_, AmbientColor>>>::fetch(res);
        ambient_color.map_or([0.0, 0.0, 0.0].into(), |c| {
            let (r, g, b, _) = c.0.into_components();
            [r, g, b].into()
        })
    }
}
