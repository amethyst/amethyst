//! Simple shaded pass

use amethyst_assets::AssetStorage;
use amethyst_core::specs::prelude::{Entities, Join, Read, ReadExpect, ReadStorage};
use amethyst_core::transform::GlobalTransform;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use light::Light;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::shaded_util::{set_light_args, setup_light_buffers};
use pass::skinning::{create_skinning_effect, setup_skinning_buffers};
use pass::util::{draw_mesh, get_camera, setup_textures, setup_vertex_args};
use pipe::pass::{Pass, PassData};
use pipe::{DepthMode, Effect, NewEffect};
use resources::AmbientColor;
use skinning::JointTransforms;
use tex::Texture;
use types::{Encoder, Factory};
use vertex::{Attributes, Normal, Position, Separate, TexCoord, VertexFormat};
use visibility::Visibility;

static ATTRIBUTES: [Attributes<'static>; 3] = [
    Separate::<Position>::ATTRIBUTES,
    Separate::<Normal>::ATTRIBUTES,
    Separate::<TexCoord>::ATTRIBUTES,
];

/// Draw mesh with simple lighting technique
#[derive(Default, Clone, Debug, PartialEq)]
pub struct DrawShadedSeparate {
    skinning: bool,
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl DrawShadedSeparate {
    /// Create instance of `DrawShaded` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable vertex skinning
    pub fn with_vertex_skinning(mut self) -> Self {
        self.skinning = true;
        self
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }
}

impl<'a> PassData<'a> for DrawShadedSeparate {
    type Data = (
        Entities<'a>,
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AmbientColor>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        ReadExpect<'a, MaterialDefaults>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Light>,
        ReadStorage<'a, JointTransforms>,
    );
}

impl Pass for DrawShadedSeparate {
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        debug!("Building shaded pass");
        let mut builder = if self.skinning {
            create_skinning_effect(effect, FRAG_SRC)
        } else {
            effect.simple(VERT_SRC, FRAG_SRC)
        };
        debug!("Effect compiled, adding vertex/uniform buffers");
        builder
            .with_raw_vertex_buffer(
                Separate::<Position>::ATTRIBUTES,
                Separate::<Position>::size() as ElemStride,
                0,
            )
            .with_raw_vertex_buffer(
                Separate::<Normal>::ATTRIBUTES,
                Separate::<Normal>::size() as ElemStride,
                0,
            )
            .with_raw_vertex_buffer(
                Separate::<TexCoord>::ATTRIBUTES,
                Separate::<TexCoord>::size() as ElemStride,
                0,
            );
        if self.skinning {
            setup_skinning_buffers(&mut builder);
        }
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
        setup_textures(&mut builder, &TEXTURES);
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (
            entities,
            active,
            camera,
            ambient,
            mesh_storage,
            tex_storage,
            material_defaults,
            visibility,
            mesh,
            material,
            global,
            light,
            joints,
        ): <Self as PassData<'a>>::Data,
    ) {
        trace!("Drawing shaded pass");
        let camera = get_camera(active, &camera, &global);

        set_light_args(effect, encoder, &light, &global, &ambient, camera);

        match visibility {
            None => for (entity, mesh, material, global) in
                (&*entities, &mesh, &material, &global).join()
            {
                draw_mesh(
                    encoder,
                    effect,
                    self.skinning,
                    mesh_storage.get(mesh),
                    joints.get(entity),
                    &tex_storage,
                    Some(material),
                    &material_defaults,
                    camera,
                    Some(global),
                    &ATTRIBUTES,
                    &TEXTURES,
                );
            },
            Some(ref visibility) => {
                for (entity, mesh, material, global, _) in (
                    &*entities,
                    &mesh,
                    &material,
                    &global,
                    &visibility.visible_unordered,
                ).join()
                {
                    draw_mesh(
                        encoder,
                        effect,
                        self.skinning,
                        mesh_storage.get(mesh),
                        joints.get(entity),
                        &tex_storage,
                        Some(material),
                        &material_defaults,
                        camera,
                        Some(global),
                        &ATTRIBUTES,
                        &TEXTURES,
                    );
                }

                for entity in &visibility.visible_ordered {
                    if let Some(mesh) = mesh.get(*entity) {
                        draw_mesh(
                            encoder,
                            effect,
                            self.skinning,
                            mesh_storage.get(mesh),
                            joints.get(*entity),
                            &tex_storage,
                            material.get(*entity),
                            &material_defaults,
                            camera,
                            global.get(*entity),
                            &ATTRIBUTES,
                            &TEXTURES,
                        );
                    }
                }
            }
        }
    }
}
