//! Utilities for skinning

use std::mem;

use gfx::pso::buffer::ElemStride;

use crate::{
    mesh::Mesh,
    pass::util::set_attribute_buffers,
    pipe::{Effect, EffectBuilder, NewEffect},
    skinning::{JointIds, JointWeights},
    vertex::{Attributes, Separate, VertexFormat},
};

static VERT_SKIN_SRC: &[u8] = include_bytes!("shaders/vertex/skinned.glsl");
static ATTRIBUTES: [Attributes<'static>; 2] = [
    Separate::<JointIds>::ATTRIBUTES,
    Separate::<JointWeights>::ATTRIBUTES,
];

pub(crate) fn create_skinning_effect<'a>(
    effect: NewEffect<'a>,
    frag: &'a [u8],
) -> EffectBuilder<'a> {
    effect.simple(VERT_SKIN_SRC, frag)
}

pub(crate) fn setup_skinning_buffers<'a>(builder: &mut EffectBuilder<'a>) {
    builder
        .with_raw_vertex_buffer(
            Separate::<JointIds>::ATTRIBUTES,
            Separate::<JointIds>::size() as ElemStride,
            0,
        )
        .with_raw_vertex_buffer(
            Separate::<JointWeights>::ATTRIBUTES,
            Separate::<JointWeights>::size() as ElemStride,
            0,
        )
        .with_raw_constant_buffer("JointTransforms", mem::size_of::<[[f32; 4]; 4]>(), 100);
}

pub fn set_skinning_buffers(effect: &mut Effect, mesh: &Mesh) -> bool {
    set_attribute_buffers(effect, mesh, &ATTRIBUTES)
}
