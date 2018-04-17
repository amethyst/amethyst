use std::mem;

use amethyst_assets::AssetStorage;
use amethyst_core::cgmath::{Matrix4, One, SquareMatrix};
use amethyst_core::specs::{Fetch, Join, ReadStorage};
use amethyst_core::GlobalTransform;

use cam::{ActiveCamera, Camera};
use mesh::Mesh;
use mtl::{Material, MaterialDefaults};
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
}

pub(crate) fn add_textures(
    effect: &mut Effect,
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
        &tex_storage,
        material.unwrap(),
        &material_defaults.0,
        textures,
    );

    effect.draw(mesh.slice(), encoder);
    effect.clear();
}

pub(crate) fn get_camera<'a>(
    active: Option<Fetch<'a, ActiveCamera>>,
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
