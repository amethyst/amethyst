//! Simple shaded pass

use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use amethyst_core::transform::Transform;
use gfx::pso::buffer::ElemStride;
use specs::{Fetch, Join, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use light::Light;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::shaded_util::{set_light_args, setup_light_buffers};
use pass::util::{add_textures, set_attribute_buffers, set_vertex_args, setup_textures,
                 setup_vertex_args};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassData};
use resources::AmbientColor;
use tex::Texture;
use types::{Encoder, Factory};
use vertex::{Normal, Position, Query, TexCoord};

/// Draw mesh with simple lighting technique
/// `V` is `VertexFormat`
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, Normal, TexCoord)>"))]
pub struct DrawShaded<V> {
    _pd: PhantomData<V>,
}

impl<V> DrawShaded<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    /// Create instance of `DrawShaded` pass
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a, V> PassData<'a> for DrawShaded<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    type Data = (
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Fetch<'a, AmbientColor>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Light>,
    );
}

impl<V> Pass for DrawShaded<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
        setup_textures(&mut builder, &TEXTURES);
        builder
            .with_output("out_color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (
            active,
            camera,
            ambient,
            mesh_storage,
            tex_storage,
            material_defaults,
            mesh,
            material,
            global,
            light,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera: Option<(&Camera, &Transform)> = active
            .and_then(|a| {
                let cam = camera.get(a.entity);
                let transform = global.get(a.entity);
                cam.into_iter().zip(transform.into_iter()).next()
            })
            .or_else(|| (&camera, &global).join().next());

        set_light_args(effect, encoder, &light, &ambient, camera);

        for (mesh, material, global) in (&mesh, &material, &global).join() {
            let mesh = match mesh_storage.get(mesh) {
                Some(mesh) => mesh,
                None => continue,
            };
            if !set_attribute_buffers(effect, mesh, &[V::QUERIED_ATTRIBUTES]) {
                continue;
            }

            set_vertex_args(effect, encoder, camera, global);
            add_textures(
                effect,
                &tex_storage,
                material,
                &material_defaults.0,
                &TEXTURES,
            );

            effect.draw(mesh.slice(), encoder);
            effect.clear();
        }
    }
}
