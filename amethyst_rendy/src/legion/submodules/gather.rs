//! Helper gatherer structures for collecting information about the world.
use crate::{
    camera::Camera,
    legion::camera::ActiveCamera,
    pod::{self, IntoPod},
    resources::AmbientColor,
};
use amethyst_core::{
    legion::{transform::components::*, *},
    math::{convert, Matrix4, Vector3},
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
    pub fn gather_camera_entity(state: &LegionState) -> Option<Entity> {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_camera (1st)");

        // TODO: we do not support active camera atm because of migration

        <(Read<Camera>, Read<LocalToWorld>)>::query()
            .iter_entities(&state.world)
            .nth(0)
            .map(|(e, _)| e)
    }

    /// Collect `ActiveCamera` and `Camera` instances from the provided resource storage and selects
    /// the appropriate camera to use for projection, and returns the camera position and extracted
    /// projection matrix.
    ///
    /// The matrix returned is the camera's `Projection` matrix and the camera `Transform::global_view_matrix`
    pub fn gather(state: &LegionState) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_cameras");

        let defcam = Camera::standard_3d(1.0, 1.0);
        let identity = LocalToWorld::identity();

        let active_camera = state.resources.get::<ActiveCamera>();
        let mut camera_query = <(Read<Camera>, Read<LocalToWorld>)>::query();

        let active_camera = active_camera
            .and_then(|active_camera| {
                active_camera
                    .entity
                    .and_then(|e| {
                        camera_query
                            .iter_entities(&state.world)
                            .find(|(camera_entity, (_, _))| *camera_entity == e)
                            .map(|(_, (camera, global_matrix))| (camera, global_matrix))
                    })
                    .or_else(|| {
                        camera_query
                            .iter_entities(&state.world)
                            .nth(0)
                            .map(|(e, (camera, global_matrix))| (camera, global_matrix))
                    })
            })
            .or_else(|| None);

        let (position, view_matrix, projection) =
            if let Some((camera, transform)) = active_camera.as_ref() {
                (transform.column(3).xyz(), &**transform, camera.as_matrix())
            } else {
                (identity.column(3).xyz(), &identity, defcam.as_matrix())
            };

        let camera_position = convert::<_, Vector3<f32>>(position).into_pod();

        let inverse_view_matrix = view_matrix.try_inverse().unwrap();

        let proj_view: [[f32; 4]; 4] = ((*projection) * inverse_view_matrix).into();
        let proj: [[f32; 4]; 4] = (*projection).into();
        let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(inverse_view_matrix).into();

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
    pub fn gather(world: &LegionState) -> vec3 {
        let ambient_color = world.resources.get::<AmbientColor>();

        ambient_color.map_or([0.0, 0.0, 0.0].into(), |c| {
            let (r, g, b, _) = c.0.into_components();
            [r, g, b].into()
        })
    }
}
