use std::mem;

use amethyst_core::Transform;
use gfx::traits::Pod;
use specs::{Join, ReadStorage};

use cam::Camera;
use light::{DirectionalLight, Light, PointLight};
use pipe::{Effect, EffectBuilder};
use resources::AmbientColor;
use types::Encoder;

fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 1.0]
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct FragmentArgs {
    point_light_count: i32,
    directional_light_count: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct PointLightPod {
    position: [f32; 4],
    color: [f32; 4],
    intensity: f32,
    _pad: [f32; 3],
}

unsafe impl Pod for PointLightPod {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct DirectionalLightPod {
    color: [f32; 4],
    direction: [f32; 4],
}

unsafe impl Pod for DirectionalLightPod {}

pub(crate) fn set_light_args(
    effect: &mut Effect,
    encoder: &mut Encoder,
    light: &ReadStorage<Light>,
    ambient: &AmbientColor,
    camera: Option<(&Camera, &Transform)>,
) {
    let point_lights: Vec<PointLightPod> = light
        .join()
        .filter_map(|light| {
            if let Light::Point(ref light) = *light {
                Some(PointLightPod {
                    position: pad(light.center.into()),
                    color: pad(light.color.into()),
                    intensity: light.intensity,
                    _pad: [0.0; 3],
                })
            } else {
                None
            }
        })
        .collect();

    let directional_lights: Vec<DirectionalLightPod> = light
        .join()
        .filter_map(|light| {
            if let Light::Directional(ref light) = *light {
                Some(DirectionalLightPod {
                    color: pad(light.color.into()),
                    direction: pad(light.direction.into()),
                })
            } else {
                None
            }
        })
        .collect();

    let fragment_args = FragmentArgs {
        point_light_count: point_lights.len() as i32,
        directional_light_count: directional_lights.len() as i32,
    };

    effect.update_constant_buffer("FragmentArgs", &fragment_args, encoder);
    effect.update_buffer("PointLights", &point_lights[..], encoder);
    effect.update_buffer("DirectionalLights", &directional_lights[..], encoder);

    effect.update_global("ambient_color", Into::<[f32; 3]>::into(*ambient.as_ref()));

    effect.update_global(
        "camera_position",
        camera
            .as_ref()
            .map(|&(_, ref trans)| [trans.0[3][0], trans.0[3][1], trans.0[3][2]])
            .unwrap_or([0.0; 3]),
    );
}

pub(crate) fn setup_light_buffers(builder: &mut EffectBuilder) {
    builder
        .with_raw_constant_buffer("FragmentArgs", mem::size_of::<FragmentArgs>(), 1)
        .with_raw_constant_buffer("PointLights", mem::size_of::<PointLight>(), 128)
        .with_raw_constant_buffer("DirectionalLights", mem::size_of::<DirectionalLight>(), 16)
        .with_raw_global("ambient_color")
        .with_raw_global("camera_position");
}
