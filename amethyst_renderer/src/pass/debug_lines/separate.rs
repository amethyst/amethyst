//! Debug lines pass

use super::*;
use amethyst_assets::AssetStorage;
use amethyst_core::specs::prelude::{Entities, Join, Read, ReadStorage};
use amethyst_core::transform::GlobalTransform;
use cam::{ActiveCamera, Camera};
use error::Result;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use mesh::{Mesh, MeshHandle};
use pass::util::{
    draw_mesh, get_camera, set_attribute_buffers, set_vertex_args, setup_vertex_args,
};
use pipe::pass::{Pass, PassData};
use pipe::{DepthMode, Effect, NewEffect};
use types::{Encoder, Factory};
use vertex::{Attributes, Normal, Position, Separate, VertexFormat};
use visibility::Visibility;

static ATTRIBUTES: [Attributes<'static>; 2] = [
    Separate::<Position>::ATTRIBUTES,
    Separate::<Normal>::ATTRIBUTES,
];

/// Draw a bunch of instanced debugging lines
#[derive(Default, Clone, Debug, PartialEq)]
pub struct DrawDebugLinesSeparate {
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl DrawDebugLinesSeparate {
    /// Create instance of `DrawDebugLines` pass
    pub fn new() -> Self {
        Default::default()
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

impl<'a> PassData<'a> for DrawDebugLinesSeparate {
    type Data = (
        Entities<'a>,
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<Mesh>>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, GlobalTransform>,
    );
}

impl Pass for DrawDebugLinesSeparate {
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        debug!("Building shaded pass");
        let mut builder = effect.geom(VERT_SRC, GEOM_SRC, FRAG_SRC);

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
            );
        setup_vertex_args(&mut builder);
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
        (entities, active, camera, mesh_storage, visibility, mesh, global): <Self as PassData<
            'a,
        >>::Data,
    ) {
        trace!("Drawing shaded pass");
        let camera = get_camera(active, &camera, &global);

        match visibility {
            None => for (entity, mesh, global) in (&*entities, &mesh, &global).join() {
                let mesh = match mesh_storage.get(mesh) {
                    Some(mesh) => mesh,
                    None => return,
                };

                if !set_attribute_buffers(effect, mesh, &ATTRIBUTES) {
                    effect.clear();
                    return;
                }

                set_vertex_args(effect, encoder, camera, global);

                effect.draw(mesh.slice(), encoder);
                effect.clear();
            },
            Some(ref visibility) => {
                for (entity, mesh, global, _) in
                    (&*entities, &mesh, &global, &visibility.visible_unordered).join()
                {
                    let mesh = match mesh_storage.get(mesh) {
                        Some(mesh) => mesh,
                        None => return,
                    };

                    if !set_attribute_buffers(effect, mesh, &ATTRIBUTES) {
                        effect.clear();
                        return;
                    }

                    set_vertex_args(effect, encoder, camera, global);

                    effect.draw(mesh.slice(), encoder);
                    effect.clear();
                }
            }
        }
    }
}
