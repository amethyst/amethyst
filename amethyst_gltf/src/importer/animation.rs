use std::collections::HashMap;

use amethyst_animation::{
    Animation, AnimationSet, InterpolationFunction, InterpolationPrimitive, Sampler,
    SamplerPrimitive, TransformChannel,
};
use amethyst_assets::{
    distill_importer::{Error, ImportOp, ImportedAsset},
    make_handle,
};
use amethyst_core::{
    ecs::{Entity, World},
    math::{convert, Vector3, Vector4},
    Transform,
};
use fnv::FnvHashMap;
use gltf::{buffer::Data, iter};

use crate::importer::GltfImporterState;

pub fn load_animations(
    animations: iter::Animations<'_>,
    buffers: &Vec<Data>,
    node_map: &HashMap<usize, Entity>,
    op: &mut ImportOp,
    state: &mut GltfImporterState,
    world: &mut World,
    animation_entity: &Entity,
) -> Vec<ImportedAsset> {
    if state.animation_sampler_uuids.is_none() {
        state.animation_sampler_uuids = Some(HashMap::new());
    }

    if state.animation_uuids.is_none() {
        state.animation_uuids = Some(HashMap::new());
    }

    let mut asset_accumulator = Vec::new();
    let mut animations_accumulator = FnvHashMap::default();

    animations.for_each(|animation| {
        let samplers = load_samplers(&animation, buffers).unwrap_or_else(|_| {
            panic!(
                "Animation sampling loading didn't work for animation {:?}",
                animation.index()
            )
        });
        if samplers
            .iter()
            .any(|sampler| node_map.contains_key(&sampler.0))
        {
            let mut nodes = Vec::new();
            for (sampler_index, (node_index, channel, sampler)) in samplers.iter().enumerate() {
                let sampler_asset_id = *state
                    .animation_sampler_uuids
                    .as_mut()
                    .expect("Animation Samplers hashmap didn't work")
                    .entry(format!("{}_{}", animation.index(), sampler_index))
                    .or_insert_with(|| op.new_asset_uuid());
                asset_accumulator.push(ImportedAsset {
                    id: sampler_asset_id,
                    search_tags: vec![],
                    build_deps: vec![],
                    load_deps: vec![],
                    build_pipeline: None,
                    asset_data: Box::new(sampler.clone()),
                });
                nodes.push((*node_index, *channel, make_handle(sampler_asset_id)));
            }

            let animation_asset_id = *state
                .animation_uuids
                .as_mut()
                .expect("Animations hashmap didn't work")
                .entry(format!("{}", animation.index()))
                .or_insert_with(|| op.new_asset_uuid());

            asset_accumulator.push(ImportedAsset {
                id: animation_asset_id,
                search_tags: vec![],
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(Animation::<Transform> { nodes }),
            });

            animations_accumulator.insert(animation.index(), make_handle(animation_asset_id));
        }
    });

    world
        .entry(*animation_entity)
        .expect("Unreachable: `animation_entity` is initialized previously")
        .add_component(AnimationSet::<usize, Transform> {
            animations: animations_accumulator,
        });

    asset_accumulator
}

fn load_samplers(
    animation: &gltf::Animation<'_>,
    buffers: &Vec<Data>,
) -> Result<Vec<(usize, TransformChannel, Sampler<SamplerPrimitive<f32>>)>, Error> {
    Ok(animation
        .channels()
        .map(|ref channel| load_channel(channel, buffers))
        .collect::<Result<Vec<_>, Error>>()
        .expect("Animation channel loading didn't work"))
}

fn load_channel(
    channel: &gltf::animation::Channel<'_>,
    buffers: &Vec<Data>,
) -> Result<(usize, TransformChannel, Sampler<SamplerPrimitive<f32>>), Error> {
    use gltf::animation::util::ReadOutputs::*;
    let sampler = channel.sampler();
    let target = channel.target();

    let reader = channel.reader(|buffer| {
        Some(
            buffers
                .get(buffer.index())
                .expect("Error while reading skin buffer")
                .0
                .as_slice(),
        )
    });

    let input = reader
        .read_inputs()
        .ok_or(Error::Custom("Channel missing inputs".to_string()))?
        .collect();
    let node_index = target.node().index();

    match reader
        .read_outputs()
        .ok_or(Error::Custom("Channel missing outputs".to_string()))?
    {
        Translations(translations) => {
            Ok((
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
            ))
        }
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
        Scales(scales) => {
            Ok((
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
            ))
        }
        MorphTargetWeights(_) => Err(Error::Custom("Not implemented".to_string())),
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
