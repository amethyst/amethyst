use std::mem;

use gfx_core::state::{Blend, ColorMask};
use glsl_layout::*;
use log::error;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    alga::general::SubsetOf,
    ecs::prelude::{Join, Read, ReadStorage},
    math::{convert, Matrix4, Orthographic3, RealField},
    Transform,
};

use crate::{
    cam::{ActiveCamera, Camera},
    mesh::Mesh,
    mtl::{Material, MaterialDefaults, TextureOffset},
    pass::set_skinning_buffers,
    pipe::{DepthMode, Effect, EffectBuilder},
    resources::ScreenDimensions,
    screen_space::ScreenSpaceSettings,
    skinning::JointTransforms,
    tex::Texture,
    types::Encoder,
    vertex::Attributes,
    Rgba,
};

use gfx::traits::Pod;

pub(crate) enum TextureType {
    Albedo,
    Emission,
    Normal,
    Metallic,
    Roughness,
    AmbientOcclusion,
    Caveat,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct ViewArgs {
    proj: mat4,
    view: mat4,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct VertexArgs {
    proj: mat4,
    view: mat4,
    model: mat4,
    rgba: vec4,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct TextureOffsetPod {
    pub u_offset: vec2,
    pub v_offset: vec2,
}

impl TextureOffsetPod {
    pub(crate) fn from_offset(offset: &TextureOffset) -> Self {
        TextureOffsetPod {
            u_offset: [offset.u.0, offset.u.1].into(),
            v_offset: [offset.v.0, offset.v.1].into(),
        }
    }
}

pub(crate) fn set_attribute_buffers(
    effect: &mut Effect,
    mesh: &Mesh,
    attributes: &[Attributes<'static>],
) -> bool {
    #[cfg(feature = "profiler")]
    profile_scope!("render_setattributebuffers");
    for attr in attributes.iter() {
        match mesh.buffer(attr) {
            Some(vbuf) => effect.data.vertex_bufs.push(vbuf.clone()),
            None => {
                error!(
                    "Required vertex attribute buffer with format {:?} missing in mesh",
                    attr
                );
                return false;
            }
        }
    }
    true
}

pub(crate) fn add_texture(effect: &mut Effect, texture: &Texture) {
    effect.data.textures.push(texture.view().clone());
    effect.data.samplers.push(texture.sampler().clone());
}

pub(crate) fn setup_textures(builder: &mut EffectBuilder<'_>, types: &[TextureType]) {
    use self::TextureType::*;

    #[cfg(feature = "profiler")]
    profile_scope!("render_setuptextures");

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
        add_texture(effect, texture.expect("Texture missing in asset storage"));
    }
    set_texture_offsets(effect, encoder, material, types);
}

pub(crate) fn setup_texture_offsets(builder: &mut EffectBuilder<'_>, types: &[TextureType]) {
    use self::TextureType::*;

    #[cfg(feature = "profiler")]
    profile_scope!("render_setuptextureoffsets");

    for ty in types {
        match *ty {
            Albedo => builder.with_raw_constant_buffer(
                "AlbedoOffset",
                mem::size_of::<<TextureOffsetPod as Uniform>::Std140>(),
                1,
            ),
            Emission => builder.with_raw_constant_buffer(
                "EmissionOffset",
                mem::size_of::<<TextureOffsetPod as Uniform>::Std140>(),
                1,
            ),
            Normal => builder.with_raw_constant_buffer(
                "NormalOffset",
                mem::size_of::<<TextureOffsetPod as Uniform>::Std140>(),
                1,
            ),
            Metallic => builder.with_raw_constant_buffer(
                "MetallicOffset",
                mem::size_of::<<TextureOffsetPod as Uniform>::Std140>(),
                1,
            ),
            Roughness => builder.with_raw_constant_buffer(
                "RoughnessOffset",
                mem::size_of::<<TextureOffsetPod as Uniform>::Std140>(),
                1,
            ),
            AmbientOcclusion => builder.with_raw_constant_buffer(
                "AmbientOcclusionOffset",
                mem::size_of::<<TextureOffsetPod as Uniform>::Std140>(),
                1,
            ),
            Caveat => builder.with_raw_constant_buffer(
                "CaveatOffset",
                mem::size_of::<<TextureOffsetPod as Uniform>::Std140>(),
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
                &TextureOffsetPod::from_offset(&material.albedo_offset).std140(),
                encoder,
            ),
            Emission => effect.update_constant_buffer(
                "EmissionOffset",
                &TextureOffsetPod::from_offset(&material.emission_offset).std140(),
                encoder,
            ),
            Normal => effect.update_constant_buffer(
                "NormalOffset",
                &TextureOffsetPod::from_offset(&material.normal_offset).std140(),
                encoder,
            ),
            Metallic => effect.update_constant_buffer(
                "MetallicOffset",
                &TextureOffsetPod::from_offset(&material.metallic_offset).std140(),
                encoder,
            ),
            Roughness => effect.update_constant_buffer(
                "RoughnessOffset",
                &TextureOffsetPod::from_offset(&material.roughness_offset).std140(),
                encoder,
            ),
            AmbientOcclusion => effect.update_constant_buffer(
                "AmbientOcclusionOffset",
                &TextureOffsetPod::from_offset(&material.ambient_occlusion_offset).std140(),
                encoder,
            ),
            Caveat => effect.update_constant_buffer(
                "CaveatOffset",
                &TextureOffsetPod::from_offset(&material.caveat_offset).std140(),
                encoder,
            ),
        };
    }
}

pub(crate) fn setup_vertex_args(builder: &mut EffectBuilder<'_>) {
    #[cfg(feature = "profiler")]
    profile_scope!("render_setupvertexargs");

    builder.with_raw_constant_buffer(
        "VertexArgs",
        mem::size_of::<<VertexArgs as Uniform>::Std140>(),
        1,
    );
}

/// Sets the vertex argument in the constant buffer.
pub fn set_vertex_args<N: RealField + SubsetOf<f32>>(
    effect: &mut Effect,
    encoder: &mut Encoder,
    camera: Option<(&Camera, &Transform<N>)>,
    global_matrix: &Matrix4<N>,
    rgba: Rgba,
) {
    let model: [[f32; 4]; 4] = convert::<Matrix4<N>, Matrix4<f32>>(*global_matrix).into();

    let vertex_args = camera
        .as_ref()
        .map(|&(ref cam, ref cam_transform)| {
            let proj: [[f32; 4]; 4] = cam.proj.into();
            let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(
                cam_transform
                    .global_matrix()
                    .try_inverse()
                    .expect("Unable to get inverse of camera transform"),
            )
            .into();
            VertexArgs {
                proj: proj.into(),
                view: view.into(),
                model: model.into(),
                rgba: rgba.into(),
            }
        })
        .unwrap_or_else(|| {
            let proj: [[f32; 4]; 4] = Matrix4::identity().into();
            let view: [[f32; 4]; 4] = Matrix4::identity().into();
            VertexArgs {
                proj: proj.into(),
                view: view.into(),
                model: model.into(),
                rgba: rgba.into(),
            }
        });
    effect.update_constant_buffer("VertexArgs", &vertex_args.std140(), encoder);
}

/// Sets the view arguments in the contant buffer.
pub fn set_view_args<N>(
    effect: &mut Effect,
    encoder: &mut Encoder,
    camera: Option<(&Camera, &Transform<N>)>,
) where
    N: RealField + SubsetOf<f32>,
{
    #[cfg(feature = "profiler")]
    profile_scope!("render_setviewargs");

    let view_args = camera
        .as_ref()
        .map(|&(ref cam, ref transform)| {
            let proj: [[f32; 4]; 4] = cam.proj.into();
            let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(
                transform
                    .global_matrix()
                    .try_inverse()
                    .expect("Unable to get inverse of camera transform"),
            )
            .into();
            ViewArgs {
                proj: proj.into(),
                view: view.into(),
            }
        })
        .unwrap_or_else(|| {
            let identity: [[f32; 4]; 4] = Matrix4::identity().into();
            ViewArgs {
                proj: identity.into(),
                view: identity.into(),
            }
        });
    effect.update_constant_buffer("ViewArgs", &view_args.std140(), encoder);
}

/// Sets the view arguments in the constant buffer using the screen dimensions.
pub fn set_view_args_screen(
    effect: &mut Effect,
    encoder: &mut Encoder,
    screen_dimensions: &ScreenDimensions,
    settings: &ScreenSpaceSettings,
) {
    #[cfg(feature = "profiler")]
    profile_scope!("render_setviewargsscreen");

    let proj: [[f32; 4]; 4] = Orthographic3::new(
        0.0,
        screen_dimensions.width(),
        0.0,
        screen_dimensions.height(),
        0.1,
        settings.max_depth,
    )
    .to_homogeneous()
    .into();
    let view_args = ViewArgs {
        proj: proj.into(),
        view: settings.view_matrix.into(),
    };
    effect.update_constant_buffer("ViewArgs", &view_args.std140(), encoder);
}

pub(crate) fn draw_mesh<N>(
    encoder: &mut Encoder,
    effect: &mut Effect,
    skinning: bool,
    mesh: Option<&Mesh>,
    joint: Option<&JointTransforms<N>>,
    tex_storage: &AssetStorage<Texture>,
    material: Option<&Material>,
    material_defaults: &MaterialDefaults,
    rgba: Option<&Rgba>,
    camera: Option<(&Camera, &Transform<N>)>,
    transform: Option<&Transform<N>>,
    attributes: &[Attributes<'static>],
    textures: &[TextureType],
) where
    N: RealField + SubsetOf<f32> + Pod,
{
    #[cfg(feature = "profiler")]
    profile_scope!("render_drawmesh");

    // Return straight away if some parameters are none
    // Consider changing function signature?
    let (mesh, material, transform) = match (mesh, material, transform) {
        (Some(v1), Some(v2), Some(v3)) => (v1, v2, v3),
        _ => return,
    };

    if !set_attribute_buffers(effect, mesh, attributes)
        || (skinning && !set_skinning_buffers(effect, mesh))
    {
        effect.clear();
        return;
    }

    set_vertex_args(
        effect,
        encoder,
        camera,
        transform.global_matrix(),
        rgba.cloned().unwrap_or(Rgba::WHITE),
    );

    if skinning {
        if let Some(joint) = joint {
            effect.update_buffer("JointTransforms", &joint.matrices[..], encoder);
        }
    }

    add_textures(
        effect,
        encoder,
        &tex_storage,
        material,
        &material_defaults.0,
        textures,
    );

    effect.draw(mesh.slice(), encoder);
    effect.clear();
}

/// Returns the main camera and its `Transform<N>`
pub fn get_camera<'a, N>(
    active: Read<'a, ActiveCamera>,
    camera: &'a ReadStorage<'a, Camera>,
    transform: &'a ReadStorage<'a, Transform<N>>,
) -> Option<(&'a Camera, &'a Transform<N>)>
where
    N: RealField,
{
    #[cfg(feature = "profiler")]
    profile_scope!("render_getcamera");

    active
        .entity
        .and_then(|entity| {
            let cam = camera.get(entity);
            let transform = transform.get(entity);
            cam.into_iter().zip(transform.into_iter()).next()
        })
        .or_else(|| (camera, transform).join().next())
}

pub fn default_transparency() -> Option<(ColorMask, Blend, Option<DepthMode>)> {
    Some((
        ColorMask::all(),
        crate::ALPHA,
        Some(DepthMode::LessEqualWrite),
    ))
}
