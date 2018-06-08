use std::collections::HashMap;

use animation::{AnimationPrefab, AnimationSetPrefab, InterpolationFunction,
                InterpolationPrimitive, Sampler, SamplerPrimitive, TransformChannel};
use core::Transform;
use gltf;

use super::{Buffers, GltfError};

pub fn load_animations(
    gltf: &gltf::Gltf,
    buffers: &Buffers,
    node_map: &HashMap<usize, usize>,
) -> Result<AnimationSetPrefab<usize, Transform>, GltfError> {
    let mut prefab = AnimationSetPrefab::default();
    for animation in gltf.animations() {
        let anim = load_animation(&animation, buffers)?;
        if anim.samplers
            .iter()
            .any(|sampler| node_map.contains_key(&sampler.0))
        {
            prefab.animations.push((animation.index(), anim));
        }
    }
    Ok(prefab)
}

fn load_animation(
    animation: &gltf::Animation,
    buffers: &Buffers,
) -> Result<AnimationPrefab<Transform>, GltfError> {
    let mut a = AnimationPrefab::default();
    a.samplers = animation
        .channels()
        .map(|ref channel| load_channel(channel, buffers))
        .collect::<Result<Vec<_>, GltfError>>()?;
    Ok(a)
}

fn load_channel(
    channel: &gltf::animation::Channel,
    buffers: &Buffers,
) -> Result<(usize, TransformChannel, Sampler<SamplerPrimitive<f32>>), GltfError> {
    use gltf::animation::TrsProperty::*;
    use gltf_utils::AccessorIter;
    let sampler = channel.sampler();
    let target = channel.target();
    let input = AccessorIter::new(sampler.input(), buffers).collect::<Vec<f32>>();
    let node_index = target.node().index();

    match target.path() {
        Translation => {
            let output = AccessorIter::new(sampler.output(), buffers)
                .map(|t| SamplerPrimitive::Vec3(t))
                .collect::<Vec<_>>();
            Ok((
                node_index,
                TransformChannel::Translation,
                Sampler {
                    input,
                    function: map_interpolation_type(&sampler.interpolation()),
                    output,
                },
            ))
        }
        Scale => {
            let output = AccessorIter::new(sampler.output(), buffers)
                .map(|t| SamplerPrimitive::Vec3(t))
                .collect::<Vec<_>>();
            Ok((
                node_index,
                TransformChannel::Scale,
                Sampler {
                    input,
                    function: map_interpolation_type(&sampler.interpolation()),
                    output,
                },
            ))
        }
        Rotation => {
            let output = AccessorIter::<[f32; 4]>::new(sampler.output(), buffers)
                .map(|q| [q[3], q[0], q[1], q[2]].into())
                .collect::<Vec<_>>();
            // gltf quat format: [x, y, z, w], our quat format: [w, x, y, z]
            let ty = map_interpolation_type(&sampler.interpolation());
            let ty = if ty == InterpolationFunction::Linear {
                InterpolationFunction::SphericalLinear
            } else {
                ty
            };
            Ok((
                node_index,
                TransformChannel::Rotation,
                Sampler {
                    input,
                    function: ty,
                    output,
                },
            ))
        }
        Weights => Err(GltfError::NotImplemented),
    }
}

fn map_interpolation_type<T>(
    ty: &gltf::animation::InterpolationAlgorithm,
) -> InterpolationFunction<T>
where
    T: InterpolationPrimitive,
{
    use gltf::animation::InterpolationAlgorithm::*;

    match *ty {
        Linear => InterpolationFunction::Linear,
        Step => InterpolationFunction::Step,
        CubicSpline => InterpolationFunction::CubicSpline,
        CatmullRomSpline => InterpolationFunction::CatmullRomSpline,
    }
}
