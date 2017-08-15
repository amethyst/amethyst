//! Forward physically-based drawing pass.

use cgmath::{Matrix4, One};
use error::Result;
use gfx::pso::buffer::ElemStride;
use gfx::traits::Pod;
use light::{DirectionalLight, Light, PointLight};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::Pass;
use scene::{Model, Scene};
use std::marker::PhantomData;
use std::mem;
use types::Encoder;
use vertex::{Attribute, Normal, Position, Tangent, TextureCoord, VertexFormat, WithField};

static VERT_SRC: &[u8] = include_bytes!("shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("shaders/fragment/pbm.glsl");

/// Draw mesh without lighting
#[derive(Clone, Debug, PartialEq)]
pub struct DrawShaded<V> {
    vertex_attributes: [(&'static str, Attribute); 4],
    _pd: PhantomData<V>,
}

impl<V> DrawShaded<V>
    where V: VertexFormat +
          WithField<Position> +
          WithField<Normal> +
          WithField<Tangent> +
          WithField<TextureCoord>
{
/// Create instance of `DrawShaded` pass
    pub fn new() -> Self {
        DrawShaded {
            vertex_attributes: [
                ("position", V::attribute::<Position>()),
                ("normal", V::attribute::<Normal>()),
                ("tangent", V::attribute::<Tangent>()),
                ("tex_coord", V::attribute::<TextureCoord>()),
            ],
            _pd: PhantomData,
        }
    }
}

fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 1.0]
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct VertexArgs {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct FragmentArgs {
    point_light_count: i32,
    directional_light_count: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PointLight2 {
    position: [f32; 4],
    color: [f32; 4],
    intensity: f32,
    _pad: [f32; 3],
}

unsafe impl Pod for PointLight2 {}

impl<V: VertexFormat> Pass for DrawShaded<V> {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        effect.simple(VERT_SRC, FRAG_SRC)
            .with_raw_vertex_buffer(self.vertex_attributes.as_ref(), V::size() as ElemStride, 0)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_constant_buffer("FragmentArgs", mem::size_of::<FragmentArgs>(), 1)
            .with_raw_constant_buffer("PointLights", mem::size_of::<PointLight>(), 512)
            .with_raw_constant_buffer("DirectionalLight", mem::size_of::<DirectionalLight>(), 16)
            .with_raw_global("ambient_color")
            .with_raw_global("camera_position")
            .with_texture("roughness")
            .with_texture("caveat")
            .with_texture("metallic")
            .with_texture("ambient_occlusion")
            .with_texture("emission")
            .with_texture("normal")
            .with_texture("albedo")
            .with_output("out_color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply(&self, enc: &mut Encoder, effect: &mut Effect, scene: &Scene, model: &Model) {
        use rayon::prelude::*;

        let vertex_args = scene
            .active_camera()
            .map(|cam| {
                     VertexArgs {
                         proj: cam.proj.into(),
                         view: cam.to_view_matrix().into(),
                         model: model.pos.into(),
                     }
                 })
            .unwrap_or_else(|| {
                                VertexArgs {
                                    proj: Matrix4::one().into(),
                                    view: Matrix4::one().into(),
                                    model: model.pos.into(),
                                }
                            });
        effect.update_constant_buffer("VertexArgs", &vertex_args, enc);

        let point_lights: Vec<PointLight2> = scene.par_iter_lights()
            .filter_map(|light| {
                if let Light::Point(ref light) = *light {
                    Some(PointLight2 {
                        position: pad(light.center.into()),
                        color: pad(light.color.into()),
                        intensity: light.intensity,
                        _pad: [0.0; 3],
                    })
                } else {
                    None
                }
            })
            .collect();

        let directional_lights: Vec<DirectionalLight> = scene.par_iter_lights()
            .filter_map(|light| {
                if let Light::Directional(ref light) = *light {
                    Some(light.clone())
                } else {
                    None
                }
            })
            .collect();

        let fragment_args = FragmentArgs {
            point_light_count: point_lights.len() as i32,
            directional_light_count: directional_lights.len() as i32,
        };

        effect.update_constant_buffer("FragmentArgs", &fragment_args, enc);
        effect.update_buffer("PointLights", &point_lights[..], enc);
        effect.update_buffer("DirectionalLights", &directional_lights[..], enc);

        effect.update_global("ambient_color", [0.005; 3]);
        effect.update_global("camera_position", scene.active_camera()
                                                    .map(|cam| cam.eye.into())
                                                    .unwrap_or([0.0; 3]));
        effect.data.textures.push(model.material.roughness.view().clone());
        effect.data.samplers.push(model.material.roughness.sampler().clone());
        effect.data.textures.push(model.material.caveat.view().clone());
        effect.data.samplers.push(model.material.caveat.sampler().clone());
        effect.data.textures.push(model.material.metallic.view().clone());
        effect.data.samplers.push(model.material.metallic.sampler().clone());
        effect.data.textures.push(model.material.ambient_occlusion.view().clone());
        effect.data.samplers.push(model.material.ambient_occlusion.sampler().clone());
        effect.data.textures.push(model.material.emission.view().clone());
        effect.data.samplers.push(model.material.emission.sampler().clone());
        effect.data.textures.push(model.material.normal.view().clone());
        effect.data.samplers.push(model.material.normal.sampler().clone());
        effect.data.textures.push(model.material.albedo.view().clone());
        effect.data.samplers.push(model.material.albedo.sampler().clone());

        effect.draw(model, enc);
    }
}
