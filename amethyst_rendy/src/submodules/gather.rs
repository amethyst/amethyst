use crate::{
    camera::{ActiveCamera, Camera},
    pod::{self, IntoPod},
    resources::AmbientColor,
};
use amethyst_core::{
    ecs::{Join, Read, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use glsl_layout::*;

type Std140<T> = <T as AsStd140>::Std140;

pub struct CameraGatherer {
    pub camera_position: vec3,
    pub projview: Std140<pod::ViewArgs>,
}

impl CameraGatherer {
    pub fn gather(res: &Resources) -> Self {
        let (active_camera, cameras, global_transforms) = <(
            Option<Read<'_, ActiveCamera>>,
            ReadStorage<'_, Camera>,
            ReadStorage<'_, GlobalTransform>,
        )>::fetch(res);

        let defcam = Camera::standard_2d();
        let identity = GlobalTransform::default();

        let camera = active_camera
            .as_ref()
            .and_then(|ac| {
                cameras.get(ac.entity).map(|camera| {
                    (
                        camera,
                        global_transforms.get(ac.entity).unwrap_or(&identity),
                    )
                })
            })
            .unwrap_or_else(|| {
                (&cameras, &global_transforms)
                    .join()
                    .next()
                    .unwrap_or((&defcam, &identity))
            });

        let camera_position = (camera.1).0.column(3).xyz().into_pod();

        let proj: [[f32; 4]; 4] = camera.0.proj.into();
        let view: [[f32; 4]; 4] = (*camera.1)
            .0
            .try_inverse()
            .expect("Unable to get inverse of camera transform")
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
