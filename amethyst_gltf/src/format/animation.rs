use std::collections::HashMap;

use crate::{
    animation::{
        AnimationPrefab, AnimationSetPrefab, InterpolationFunction, InterpolationPrimitive,
        Sampler, SamplerPrimitive, TransformChannel,
    },
    core::Transform,
};

use super::{Buffers, GltfError};

pub fn load_animations(
    gltf: &gltf::Gltf,
    buffers: &Buffers,
    node_map: &HashMap<usize, usize>,
) -> Result<AnimationSetPrefab<usize, Transform>, GltfError> {
    let mut prefab = AnimationSetPrefab::default();
    for animation in gltf.animations() {
        let anim = load_animation(&animation, buffers)?;
        if anim
            .samplers
            .iter()
            .any(|sampler| node_map.contains_key(&sampler.0))
        {
            prefab.animations.push((animation.index(), anim));
        }
    }
    Ok(prefab)
}

fn load_animation(
    animation: &gltf::Animation<'_>,
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
    channel: &gltf::animation::Channel<'_>,
    buffers: &Buffers,
) -> Result<(usize, TransformChannel, Sampler<SamplerPrimitive<f32>>), GltfError> {
    use gltf::animation::util::ReadOutputs::*;
    let sampler = channel.sampler();
    let target = channel.target();

    let reader = channel.reader(|buffer| buffers.buffer(&buffer));
    let input = reader
        .read_inputs()
        .ok_or(GltfError::MissingInputs)?
        .collect();
    let node_index = target.node().index();

    match reader.read_outputs().ok_or(GltfError::MissingOutputs)? {
        Translations(translations) => Ok((
            node_index,
            TransformChannel::Translation,
            Sampler {
                input,
                function: map_interpolation_type(&sampler.interpolation()),
                output: translations.map(|t| t.into()).collect(),
            },
        )),
        Rotations(rotations) => {
            let ty = map_interpolation_type(&sampler.interpolation());
            let ty = if ty == InterpolationFunction::Linear {
                InterpolationFunction::SphericalLinear
            } else {
                ty
            };
            // gltf quat format: [x, y, z, w], our quat format: [w, x, y, z]
            Ok((
                node_index,
                TransformChannel::Rotation,
                Sampler {
                    input,
                    function: ty,
                    output: rotations
                        .into_f32()
                        .map(|q| [q[3], q[0], q[1], q[2]].into())
                        .collect(),
                },
            ))
        }
        Scales(scales) => Ok((
            node_index,
            TransformChannel::Scale,
            Sampler {
                input,
                function: map_interpolation_type(&sampler.interpolation()),
                output: scales.map(|s| s.into()).collect(),
            },
        )),
        MorphTargetWeights(_) => Err(GltfError::NotImplemented),
    }
}

fn map_interpolation_type<T>(ty: &gltf::animation::Interpolation) -> InterpolationFunction<T>
where
    T: InterpolationPrimitive,
{
    use gltf::animation::Interpolation::*;

    match *ty {
        Linear => InterpolationFunction::Linear,
        Step => InterpolationFunction::Step,
        CubicSpline => InterpolationFunction::CubicSpline,
        CatmullRomSpline => InterpolationFunction::CatmullRomSpline,
    }
}
