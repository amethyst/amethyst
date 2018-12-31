//! Flat forward drawing pass that mimics a blit.

use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    specs::prelude::{Read, ReadStorage, Write},
    transform::GlobalTransform,
};

use crate::{
    cam::{ActiveCamera, Camera},
    error::Result,
    pass::util::{add_texture, get_camera, set_view_args, setup_textures, ViewArgs},
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    tex::Texture,
    types::{Encoder, Factory, Slice},
    vertex::{Attributes, Query, VertexFormat},
};

use super::*;

/// Draws sprites and textures on a 2D quad.
///
/// This pass requires encoders to draw anything.
/// If you are using `RenderingBundle`, make sure to use it's `.with_drawflat2d_encoders(..)` method.
#[derive(Derivative, Clone, Debug)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawFlat2D {
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
    instance_data: Vec<f32>,
}

impl DrawFlat2D
where
    Self: Pass,
{
    /// Create instance of `DrawFlat2D` pass
    pub fn new() -> Self {
        Self {
            transparency: None,
            instance_data: Vec::with_capacity(1024),
        }
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

    fn attributes() -> Attributes<'static> {
        <SpriteInstance as Query<(DirX, DirY, Pos, OffsetU, OffsetV, Color, Depth)>>::QUERIED_ATTRIBUTES
    }
}

impl<'a> PassData<'a> for DrawFlat2D {
    type Data = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, GlobalTransform>,
        Write<'a, Vec<Flat2DData>>,
        Read<'a, AssetStorage<Texture>>,
    );
}

impl Pass for DrawFlat2D {
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect> {
        use std::mem;

        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder
            .without_back_face_culling()
            .with_raw_constant_buffer(
                "ViewArgs",
                mem::size_of::<<ViewArgs as Uniform>::Std140>(),
                1,
            )
            .with_raw_vertex_buffer(Self::attributes(), SpriteInstance::size() as ElemStride, 1);
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
        mut factory: Factory,
        (active, camera, global, mut buffer, tex_storage): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        use gfx::{
            buffer,
            memory::{Bind, Typed},
            Factory,
        };

        // Sprite vertex shader
        set_view_args(effect, encoder, camera);

        // We might be able to improve performance here if we
        // preallocate the maximum needed capacity. We need to
        // iterate over the sprites though to find out the longest
        // chain of sprites with the same texture, so we would need
        // to check if it actually results in an improvement over just
        // doing the allocations.
        let instance_data = &mut self.instance_data;
        let mut num_instances = 0;
        let num_quads = buffer.len();
        let mut current_tex_id = buffer.first().map(|d| d.texture.id()).unwrap_or(0);

        for (i, quad) in buffer.iter().enumerate() {
            let Flat2DData {
                dir_x,
                dir_y,
                pos,
                uv_left,
                uv_right,
                uv_bottom,
                uv_top,
                texture,
                tint,
                ..
            } = quad;
            instance_data.extend(&[
                dir_x.x, dir_x.y, dir_y.x, dir_y.y, pos.x, pos.y, *uv_left, *uv_right, *uv_bottom,
                *uv_top, tint.0, tint.1, tint.2, tint.3, pos.z,
            ]);
            num_instances += 1;

            // Need to flush outstanding draw calls due to state switch (texture).
            //
            // 1. We are at the last sprite and want to submit all pending work.
            // 2. The next sprite will use a different texture triggering a flush.
            let need_flush = i >= num_quads - 1 || current_tex_id != texture.id();
            current_tex_id = texture.id();

            if need_flush {
                if let Some(texture) = tex_storage.get(texture) {
                    add_texture(effect, texture);

                    let vbuf = factory
                        .create_buffer_immutable(
                            &instance_data,
                            buffer::Role::Vertex,
                            Bind::empty(),
                        )
                        .expect("Unable to create immutable buffer for `DrawFlat2D`");

                    for _ in DrawFlat2D::attributes() {
                        effect.data.vertex_bufs.push(vbuf.raw().clone());
                    }
                    effect.draw(
                        &Slice {
                            start: 0,
                            end: 6,
                            base_vertex: 0,
                            instances: Some((num_instances, 0)),
                            buffer: Default::default(),
                        },
                        encoder,
                    );

                    effect.clear();
                }

                num_instances = 0;
                instance_data.clear();
            }
        }
        buffer.clear();
    }
}
