use num_traits::NumCast;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug};

use amethyst_error::Error;

use amethyst_animation::{
    AnimationPrefab, AnimationSetPrefab, InterpolationFunction, InterpolationPrimitive, Sampler,
    SamplerPrimitive, TransformChannel,
};
use amethyst_core::{math::RealField, Transform};

use super::Buffers;
use crate::error;

pub fn load_animations<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    gltf: &gltf::Gltf,
    buffers: &Buffers,
    node_map: &HashMap<usize, usize>,
) -> Result<AnimationSetPrefab<usize, Transform<N>>, Error> {
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

fn load_animation<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    animation: &gltf::Animation<'_>,
    buffers: &Buffers,
) -> Result<AnimationPrefab<Transform<N>>, Error> {
    let mut a = AnimationPrefab::default();
    a.samplers = animation
        .channels()
        .map(|ref channel| load_channel(channel, buffers))
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(a)
}

fn load_channel<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    channel: &gltf::animation::Channel<'_>,
    buffers: &Buffers,
) -> Result<(usize, TransformChannel, Sampler<SamplerPrimitive<N>>), Error> {
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
                function: map_interpolation_type(&sampler.interpolation()),
                output: translations
                    .map(|t| [t[0].into(), t[1].into(), t[2].into()].into())
                    .collect(),
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
                        .map(|q| [q[3].into(), q[0].into(), q[1].into(), q[2].into()].into())
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
                output: scales
                    .map(|s| [s[0].into(), s[1].into(), s[2].into()].into())
                    .collect(),
            },
        )),
        MorphTargetWeights(_) => Err(error::Error::NotImplemented.into()),
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
