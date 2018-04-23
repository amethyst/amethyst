use std::mem;

use amethyst_assets::AssetStorage;
use amethyst_core::GlobalTransform;
use amethyst_core::cgmath::{Matrix4, One, SquareMatrix};
use amethyst_core::specs::prelude::{Join, Read, ReadStorage};

use cam::{ActiveCamera, Camera};
use mesh::Mesh;
use mtl::{Material, MaterialDefaults, TextureOffset};
use pass::set_skinning_buffers;
use pipe::{Effect, EffectBuilder};
use skinning::JointTransforms;
use tex::Texture;
use types::Encoder;
use vertex::Attributes;

pub(crate) enum TextureType {
    Albedo,
    Emission,
    Normal,
    Metallic,
    Roughness,
    AmbientOcclusion,
    Caveat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct VertexArgs {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct TextureOffsetPod {
    u_offset: [f32; 2],
    v_offset: [f32; 2],
}

impl TextureOffsetPod {
    pub(crate) fn from_offset(offset: &TextureOffset) -> Self {
        TextureOffsetPod {
            u_offset: [offset.u.0, offset.u.1],
            v_offset: [offset.v.0, offset.v.1],
        }
    }
}

pub(crate) fn set_attribute_buffers(
    effect: &mut Effect,
    mesh: &Mesh,
    attributes: &[Attributes<'static>],
) -> bool {
    for attr in attributes.iter() {
        match mesh.buffer(attr) {
            Some(vbuf) => effect.data.vertex_bufs.push(vbuf.clone()),
            None => return false,
        }
    }
    true
}

pub(crate) fn add_texture(effect: &mut Effect, texture: &Texture) {
    effect.data.textures.push(texture.view().clone());
    effect.data.samplers.push(texture.sampler().clone());
}

pub(crate) fn setup_textures(builder: &mut EffectBuilder, types: &[TextureType]) {
    use self::TextureType::*;
    for ty in types {
        match *ty {
            Albedo => builder.with_texture("albedo"),
            Emission => builder.with_texture("emission"),
            Normal => builder.with_texture("normal"),
            Metallic => builder.with_texture("metallic"),
            Roughness => builder.with_texture("roughness"),
            AmbientOcclusion => builder.with_texture("ambient_occlusion"),
            Caveat => builder.with_texture("caveat"),
        };
    }
    setup_texture_offsets(builder, types);
}

pub(crate) fn add_textures(
    effect: &mut Effect,
    encoder: &mut Encoder,
    storage: &AssetStorage<Texture>,
    material: &Material,
    default: &Material,
    types: &[TextureType],
) {
    use self::TextureType::*;
    for ty in types {
        let texture = match *ty {
            Albedo => storage
                .get(&material.albedo)
                .or_else(|| storage.get(&default.albedo)),
            Emission => storage
                .get(&material.emission)
                .or_else(|| storage.get(&default.emission)),
            Normal => storage
                .get(&material.normal)
                .or_else(|| storage.get(&default.normal)),
            Metallic => storage
                .get(&material.metallic)
                .or_else(|| storage.get(&default.metallic)),
            Roughness => storage
                .get(&material.roughness)
                .or_else(|| storage.get(&default.roughness)),
            AmbientOcclusion => storage
                .get(&material.ambient_occlusion)
                .or_else(|| storage.get(&default.ambient_occlusion)),
            Caveat => storage
                .get(&material.caveat)
                .or_else(|| storage.get(&default.caveat)),
        };
        add_texture(effect, texture.unwrap());
    }
    set_texture_offsets(effect, encoder, material, types);
}

pub(crate) fn setup_texture_offsets(builder: &mut EffectBuilder, types: &[TextureType]) {
    use self::TextureType::*;
    for ty in types {
        match *ty {
            Albedo => builder.with_raw_constant_buffer(
                "AlbedoOffset",
                mem::size_of::<TextureOffsetPod>(),
                1,
            ),
            Emission => builder.with_raw_constant_buffer(
                "EmissionOffset",
                mem::size_of::<TextureOffsetPod>(),
                1,
            ),
            Normal => builder.with_raw_constant_buffer(
                "NormalOffset",
                mem::size_of::<TextureOffsetPod>(),
                1,
            ),
            Metallic => builder.with_raw_constant_buffer(
                "MetallicOffset",
                mem::size_of::<TextureOffsetPod>(),
                1,
            ),
            Roughness => builder.with_raw_constant_buffer(
                "RoughnessOffset",
                mem::size_of::<TextureOffsetPod>(),
                1,
            ),
            AmbientOcclusion => builder.with_raw_constant_buffer(
                "AmbientOcclusionOffset",
                mem::size_of::<TextureOffsetPod>(),
                1,
            ),
            Caveat => builder.with_raw_constant_buffer(
                "CaveatOffset",
                mem::size_of::<TextureOffsetPod>(),
                1,
            ),
        };
    }
}

pub(crate) fn set_texture_offsets(
    effect: &mut Effect,
    encoder: &mut Encoder,
    material: &Material,
    types: &[TextureType],
) {
    use self::TextureType::*;
    for ty in types {
        match *ty {
            Albedo => effect.update_constant_buffer(
                "AlbedoOffset",
                &TextureOffsetPod::from_offset(&material.albedo_offset),
                encoder,
            ),
            Emission => effect.update_constant_buffer(
                "EmissionOffset",
                &TextureOffsetPod::from_offset(&material.emission_offset),
                encoder,
            ),
            Normal => effect.update_constant_buffer(
                "NormalOffset",
                &TextureOffsetPod::from_offset(&material.normal_offset),
                encoder,
            ),
            Metallic => effect.update_constant_buffer(
                "MetallicOffset",
                &TextureOffsetPod::from_offset(&material.metallic_offset),
                encoder,
            ),
            Roughness => effect.update_constant_buffer(
                "RoughnessOffset",
                &TextureOffsetPod::from_offset(&material.roughness_offset),
                encoder,
            ),
            AmbientOcclusion => effect.update_constant_buffer(
                "AmbientOcclusionOffset",
                &TextureOffsetPod::from_offset(&material.ambient_occlusion_offset),
                encoder,
            ),
            Caveat => effect.update_constant_buffer(
                "CaveatOffset",
                &TextureOffsetPod::from_offset(&material.caveat_offset),
                encoder,
            ),
        };
    }
}

pub(crate) fn setup_vertex_args(builder: &mut EffectBuilder) {
    builder.with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1);
}

pub(crate) fn set_vertex_args(
    effect: &mut Effect,
    encoder: &mut Encoder,
    camera: Option<(&Camera, &GlobalTransform)>,
    global: &GlobalTransform,
) {
    let vertex_args = camera
        .as_ref()
        .map(|&(ref cam, ref transform)| VertexArgs {
            proj: cam.proj.into(),
            view: transform.0.invert().unwrap().into(),
            model: *global.as_ref(),
        })
        .unwrap_or_else(|| VertexArgs {
            proj: Matrix4::one().into(),
            view: Matrix4::one().into(),
            model: *global.as_ref(),
        });
    effect.update_constant_buffer("VertexArgs", &vertex_args, encoder);
}

pub(crate) fn draw_mesh(
    encoder: &mut Encoder,
    effect: &mut Effect,
    skinning: bool,
    mesh: Option<&Mesh>,
    joint: Option<&JointTransforms>,
    tex_storage: &AssetStorage<Texture>,
    material: Option<&Material>,
    material_defaults: &MaterialDefaults,
    camera: Option<(&Camera, &GlobalTransform)>,
    global: Option<&GlobalTransform>,
    attributes: &[Attributes<'static>],
    textures: &[TextureType],
) {
    let mesh = match mesh {
        Some(mesh) => mesh,
        None => return,
    };
    if material.is_none() || global.is_none() {
        return;
    }

    if !set_attribute_buffers(effect, mesh, attributes)
        || (skinning && !set_skinning_buffers(effect, mesh))
    {
        effect.clear();
        return;
    }

    set_vertex_args(effect, encoder, camera, global.unwrap());

    if skinning {
        if let Some(joint) = joint {
            effect.update_buffer("JointTransforms", &joint.matrices[..], encoder);
        }
    }

    add_textures(
        effect,
        encoder,
        &tex_storage,
        material.unwrap(),
        &material_defaults.0,
        textures,
    );

    effect.draw(mesh.slice(), encoder);
    effect.clear();
}

pub(crate) fn get_camera<'a>(
    active: Option<Read<'a, ActiveCamera>>,
    camera: &'a ReadStorage<Camera>,
    global: &'a ReadStorage<GlobalTransform>,
) -> Option<(&'a Camera, &'a GlobalTransform)> {
    active
        .and_then(|a| {
            let cam = camera.get(a.entity);
            let transform = global.get(a.entity);
            cam.into_iter().zip(transform.into_iter()).next()
        })
        .or_else(|| (camera, global).join().next())
}
