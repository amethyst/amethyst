//! Simple flat forward drawing pass.

use amethyst_assets::AssetStorage;
use amethyst_core::cgmath::{Matrix4, One, SquareMatrix};
use amethyst_core::transform::Transform;
use gfx::pso::buffer::ElemStride;
use specs::{Entities, Fetch, Join, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialAnimation, MaterialDefaults};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassData};
use tex::Texture;
use tex_animation::SpriteSheetData;
use types::{Encoder, Factory};
use vertex::{Position, Separate, TexCoord, VertexFormat};

/// Draw mesh without lighting
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawFlatSeparate;

impl DrawFlatSeparate
where
    Self: Pass,
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> PassData<'a> for DrawFlatSeparate {
    type Data = (
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, AssetStorage<SpriteSheetData>>,
        Fetch<'a, MaterialDefaults>,
        Entities<'a>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, MaterialAnimation>,
        ReadStorage<'a, Transform>,
    );
}

impl Pass for DrawFlatSeparate {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(
                Separate::<Position>::ATTRIBUTES,
                Separate::<Position>::size() as ElemStride,
                0,
            )
            .with_raw_vertex_buffer(
                Separate::<TexCoord>::ATTRIBUTES,
                Separate::<TexCoord>::size() as ElemStride,
                0,
            )
            .with_texture("albedo")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (active, camera, mesh_storage, tex_storage, sprite_sheet_storage, material_defaults, entities, mesh, material, material_animation, global):
        <Self as PassData<'b>>::Data,
    ) {
        let camera: Option<(&Camera, &Transform)> = active
            .and_then(|a| {
                let cam = camera.get(a.entity);
                let transform = global.get(a.entity);
                cam.into_iter().zip(transform.into_iter()).next()
            })
            .or_else(|| (&camera, &global).join().next());

        for (entity, mesh, material, global) in (&*entities, &mesh, &material, &global).join() {
            let mesh = match mesh_storage.get(mesh) {
                Some(mesh) => mesh,
                None => continue,
            };
            for attrs in [
                Separate::<Position>::ATTRIBUTES,
                Separate::<TexCoord>::ATTRIBUTES,
            ].iter()
            {
                match mesh.buffer(attrs) {
                    Some(vbuf) => effect.data.vertex_bufs.push(vbuf.clone()),
                    None => return,
                }
            }

            let (tex_x, tex_y, tex_w, tex_h) = material_animation
                                                .get(entity)
                                                .and_then(|anim| anim.albedo_animation.as_ref())
                                                .and_then(|anim| {
                                                    let frame = anim.current_frame(&sprite_sheet_storage);
                                                    frame.map(|f| (f.x, f.y, f.width, f.height))
                                                })
                                                .unwrap_or((0., 0., 1., 1.));

            let vertex_args = camera
                .as_ref()
                .map(|&(ref cam, ref transform)| {
                    VertexArgs {
                        proj: cam.proj.into(),
                        view: transform.0.invert().unwrap().into(),
                        model: *global.as_ref(),
                        tex_xy: [tex_x, tex_y],
                        tex_wh: [tex_w, tex_h],
                    }
                })
                .unwrap_or_else(|| {
                    VertexArgs {
                        proj: Matrix4::one().into(),
                        view: Matrix4::one().into(),
                        model: *global.as_ref(),
                        tex_xy: [tex_x, tex_y],
                        tex_wh: [tex_w, tex_h],
                    }
                });

            let albedo = tex_storage
                .get(&material.albedo)
                .or_else(|| tex_storage.get(&material_defaults.0.albedo))
                .unwrap();

            effect.update_constant_buffer("VertexArgs", &vertex_args, encoder);
            effect.data.textures.push(albedo.view().clone());
            effect.data.samplers.push(albedo.sampler().clone());

            effect.draw(mesh.slice(), encoder);
            effect.clear();
        }
    }
}
