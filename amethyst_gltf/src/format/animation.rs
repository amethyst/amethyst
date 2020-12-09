use std::collections::HashMap;

use amethyst_animation::{
    AnimationPrefab, AnimationSetPrefab, InterpolationFunction, InterpolationPrimitive, Sampler,
    SamplerPrimitive, TransformChannel,
};
use amethyst_core::{
    math::{convert, Vector3, Vector4},
    Transform,
};
use amethyst_error::Error;

use super::Buffers;
use crate::error;

pub fn load_animations(
    gltf: &gltf::Gltf,
    buffers: &Buffers,
    node_map: &HashMap<usize, usize>,
) -> Result<AnimationSetPrefab<usize, Transform>, Error> {
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
) -> Result<AnimationPrefab<Transform>, Error> {
    let mut a = AnimationPrefab::default();
    a.samplers = animation
        .channels()
        .map(|ref channel| load_channel(channel, buffers))
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(a)
}

fn load_channel(
    channel: &gltf::animation::Channel<'_>,
    buffers: &Buffers,
) -> Result<(usize, TransformChannel, Sampler<SamplerPrimitive<f32>>), Error> {
    use gltf::animation::util::ReadOutputs::*;
    let sampler = channel.sampler();
    let target = channel.target();

    let reader = channel.reader(|buffer| buffers.buffer(&buffer));
    let input = reader
        .read_inputs()
        .ok_or(error::Error::MissingInputs)?
        .collect();
    let node_index = target.node().index();

    match reader.read_outputs().ok_or(error::Error::MissingOutputs)? {
        Translations(translations) => Ok((
            node_index,
            TransformChannel::Translation,
            Sampler {
                input,
                function: map_interpolation_type(sampler.interpolation()),
                output: translations
                    .map(Vector3::from)
                    .map(|t| convert::<_, Vector3<f32>>(t).into())
                    .collect(),
            },
        )),
        Rotations(rotations) => {
            let ty = map_interpolation_type(sampler.interpolation());
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
                    output: rotations
                        .into_f32()
                        .map(Vector4::from)
                        .map(|q| convert::<_, Vector4<f32>>(q).into())
                        .collect(),
                },
            ))
        }
        Scales(scales) => Ok((
            node_index,
            TransformChannel::Scale,
            Sampler {
                input,
                function: map_interpolation_type(sampler.interpolation()),
                output: scales
                    .map(Vector3::from)
                    .map(|s| convert::<_, Vector3<f32>>(s).into())
                    .collect(),
            },
        )),
        MorphTargetWeights(_) => Err(error::Error::NotImplemented.into()),
    }
}

fn map_interpolation_type<T>(ty: gltf::animation::Interpolation) -> InterpolationFunction<T>
where
    T: InterpolationPrimitive,
{
    use gltf::animation::Interpolation::*;

    match ty {
        Linear => InterpolationFunction::Linear,
        Step => InterpolationFunction::Step,
        CubicSpline => InterpolationFunction::CubicSpline,
    }
}
