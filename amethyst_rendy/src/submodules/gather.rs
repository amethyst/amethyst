use crate::{
    camera::{ActiveCamera, Camera},
    pod::{self, IntoPod},
    resources::AmbientColor,
};
use amethyst_core::{
    ecs::{Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
    math::{convert, Matrix4, Vector3},
    transform::Transform,
};
use amethyst_window::ScreenDimensions;
use glsl_layout::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

type Std140<T> = <T as AsStd140>::Std140;

pub struct CameraGatherer {
    pub camera_position: vec3,
    pub projview: Std140<pod::ViewArgs>,
}

impl CameraGatherer {
    pub fn gather(res: &Resources) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_cameras");

        let (active_camera, cameras, transforms, dimensions) = <(
            Option<Read<'_, ActiveCamera>>,
            ReadStorage<'_, Camera>,
            ReadStorage<'_, Transform>,
            ReadExpect<'_, ScreenDimensions>,
        )>::fetch(res);

        let defcam = Camera::standard_2d(dimensions.width(), dimensions.height());
        let identity = Transform::default();

        let (camera, transform) = active_camera
            .as_ref()
            .and_then(|ac| {
                cameras
                    .get(ac.entity)
                    .map(|camera| (camera, transforms.get(ac.entity).unwrap_or(&identity)))
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
        let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(
            transform.view_matrix()
        )
        .into();

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

pub struct AmbientGatherer;
impl AmbientGatherer {
    pub fn gather(res: &Resources) -> vec3 {
        let ambient_color = <Option<Read<'_, AmbientColor>>>::fetch(res);
        ambient_color.map_or([0.0, 0.0, 0.0].into(), |c| {
            let (r, g, b, _) = c.0.into_components();
            [r, g, b].into()
        })
    }
}
