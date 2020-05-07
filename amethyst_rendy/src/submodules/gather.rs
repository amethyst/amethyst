//! Helper gatherer structures for collecting information about the world.
use crate::{
    camera::{ActiveCamera, Camera},
    pod::{self, IntoPod},
    resources::AmbientColor,
};
use amethyst_core::{
    ecs::prelude::*,
    math::{convert, Matrix4, Vector3},
    transform::LocalToWorld,
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
    pub fn gather_camera_entity(world: &World, resources: &Resources) -> Option<Entity> {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_camera (1st)");

        // Get camera entity from `ActiveCamera` resource
        let active_camera = resources.get::<ActiveCamera>()
            .map(|r| r.entity)
            .flatten();

        // Find if such camera exists
        let entity = active_camera
            .and_then(|active_camera| {
                <Read<Camera>>::query()
                    .iter_entities(world)
                    .map(|(e,_)| e)
                    .find(|e| active_camera == *e)
            })
            .or_else(|| None);

        // Return active camera or fetch first available
        match entity {
            Some(entity) => Some(entity),
            None => {
                // Fetch first available camera
                <Read<Camera>>::query()
                    .iter_entities(world)
                    .nth(0)
                    .map(|(e, _)| e)
            }
        }
    }

    /// Collect `ActiveCamera` and `Camera` instances from the provided resource storage and selects
    /// the appropriate camera to use for projection, and returns the camera position and extracted
    /// projection matrix.
    ///
    /// The matrix returned is the camera's `Projection` matrix and the camera `Transform::global_view_matrix`
    pub fn gather(world: &World, resources: &Resources) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_cameras");

        let defcam = Camera::standard_2d(1.0, 1.0);
        let identity = LocalToWorld::identity();

        let camera_entity = Self::gather_camera_entity(world, resources);

        let camera = camera_entity.map(|e| {
            world.get_component::<Camera>(e)
        }).flatten();
        let camera = camera.as_deref().unwrap_or(&defcam);

        let transform = camera_entity.map(|e| {
            world.get_component::<LocalToWorld>(e)
        }).flatten();
        let transform = transform.as_deref().unwrap_or(&identity);

        let camera_position =
            convert::<_, Vector3<f32>>(transform.column(3).xyz()).into_pod();

        let proj = camera.as_matrix();
        let view = &**transform;

        let proj_view: [[f32; 4]; 4] = ((*proj) * view).into();
        let proj: [[f32; 4]; 4] = (*proj).into();
        let view: [[f32; 4]; 4] = (*view).into();

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

/// If an `AmbientColor` exists in the resources, return it - otherwise return pure white.
#[derive(Debug)]
pub struct AmbientGatherer;
impl AmbientGatherer {
    /// If an `AmbientColor` exists in the resources, return it - otherwise return pure white.
    pub fn gather(resources: &Resources) -> vec3 {
        resources
            .get::<AmbientColor>()
            .map_or([0.0, 0.0, 0.0].into(), |c| {
                let (r, g, b, _) = c.0.into_components();
                [r, g, b].into()
            })
    }
}
