use amethyst_core::specs::prelude::{Join, ReadStorage};
use amethyst_core::GlobalTransform;
use cam::Camera;
use glsl_layout::*;
use light::Light;
use pipe::{Effect, EffectBuilder};
use resources::AmbientColor;
use std::mem;
use types::Encoder;

#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct FragmentArgs {
    point_light_count: uint,
    directional_light_count: uint,
    spot_light_count: uint,
}

#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct PointLightPod {
    position: vec3,
    color: vec3,
    pad: float,
    intensity: float,
}

#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct DirectionalLightPod {
    color: vec3,
    direction: vec3,
}

#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct SpotLightPod {
    position: vec3,
    color: vec3,
    direction: vec3,
    angle: float,
    intensity: float,
    range: float,
    smoothness: float,
}

pub(crate) fn set_light_args(
    effect: &mut Effect,
    encoder: &mut Encoder,
    light: &ReadStorage<Light>,
    global: &ReadStorage<GlobalTransform>,
    ambient: &AmbientColor,
    camera: Option<(&Camera, &GlobalTransform)>,
) {
    let point_lights: Vec<_> = (light, global)
        .join()
        .filter_map(|(light, transform)| {
            if let Light::Point(ref light) = *light {
                Some(
                    PointLightPod {
                        position: transform.0.w.truncate().into(),
                        color: light.color.into(),
                        intensity: light.intensity,
                        pad: 0.0,
                    }.std140(),
                )
            } else {
                None
            }
        }).collect();

    let directional_lights: Vec<_> = light
        .join()
        .filter_map(|light| {
            if let Light::Directional(ref light) = *light {
                Some(
                    DirectionalLightPod {
                        color: light.color.into(),
                        direction: light.direction.into(),
                    }.std140(),
                )
            } else {
                None
            }
        }).collect();

    let spot_lights: Vec<_> = (light, global)
        .join()
        .filter_map(|(light, transform)| {
            if let Light::Spot(ref light) = *light {
                Some(
                    SpotLightPod {
                        position: transform.0.w.truncate().into(),
                        color: light.color.into(),
                        direction: light.direction.into(),
                        angle: light.angle.to_radians().cos(),
                        intensity: light.intensity,
                        range: light.range,
                        smoothness: light.smoothness,
                    }.std140(),
                )
            } else {
                None
            }
        }).collect();

    let fragment_args = FragmentArgs {
        point_light_count: point_lights.len() as u32,
        directional_light_count: directional_lights.len() as u32,
        spot_light_count: spot_lights.len() as u32,
    };

    effect.update_constant_buffer("FragmentArgs", &fragment_args.std140(), encoder);
    effect.update_buffer("PointLights", &point_lights[..], encoder);
    effect.update_buffer("DirectionalLights", &directional_lights[..], encoder);
    effect.update_buffer("SpotLights", &spot_lights[..], encoder);

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
        .with_raw_constant_buffer(
            "FragmentArgs",
            mem::size_of::<<FragmentArgs as Uniform>::Std140>(),
            1,
        ).with_raw_constant_buffer(
            "PointLights",
            mem::size_of::<<PointLightPod as Uniform>::Std140>(),
            128,
        ).with_raw_constant_buffer(
            "DirectionalLights",
            mem::size_of::<<DirectionalLightPod as Uniform>::Std140>(),
            16,
        ).with_raw_constant_buffer(
            "SpotLights",
            mem::size_of::<<SpotLightPod as Uniform>::Std140>(),
            128,
        ).with_raw_global("ambient_color")
        .with_raw_global("camera_position");
}
