//! Debug lines pass

use std::marker::PhantomData;

use derivative::Derivative;
use gfx::{pso::buffer::ElemStride, Primitive};
use log::{debug, trace};

use amethyst_core::{
    nalgebra as na,
    specs::{Join, Read, ReadStorage, Write, WriteStorage},
    transform::GlobalTransform,
};

use crate::{
    cam::{ActiveCamera, Camera},
    debug_drawing::{DebugLine, DebugLines, DebugLinesComponent},
    error::Result,
    mesh::Mesh,
    pass::util::{get_camera, set_attribute_buffers, set_vertex_args, setup_vertex_args},
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    types::{Encoder, Factory},
    vertex::{Color, Normal, Position, Query},
    Rgba,
};

use super::*;

/// Parameters for renderer of debug lines. The params affect all lines.
pub struct DebugLinesParams {
    /// Width of lines in units, default is 1.0 / 400.0 units
    pub line_width: f32,
}

impl Default for DebugLinesParams {
    fn default() -> Self {
        DebugLinesParams {
            line_width: 1.0 / 400.0,
        }
    }
}

/// Draw several simple lines for debugging
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
///
/// # Type Parameters:
///
/// * `V`: `VertexFormat`
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, Color, Normal)>"))]
pub struct DrawDebugLines<V> {
    _pd: PhantomData<V>,
}

impl<V> DrawDebugLines<V>
where
    V: Query<(Position, Color, Normal)>,
{
    /// Create instance of `DrawDebugLines` pass
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a, V> PassData<'a> for DrawDebugLines<V>
where
    V: Query<(Position, Color, Normal)>,
{
    type Data = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, GlobalTransform>,
        WriteStorage<'a, DebugLinesComponent>, // DebugLines components
        Option<Write<'a, DebugLines>>,         // DebugLines resource
        Read<'a, DebugLinesParams>,
    );
}

impl<V> Pass for DrawDebugLines<V>
where
    V: Query<(Position, Color, Normal)>,
{
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect> {
        debug!("Building debug lines pass");
        let mut builder = effect.geom(VERT_SRC, GEOM_SRC, FRAG_SRC);

        debug!("Effect compiled, adding vertex/uniform buffers");
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);

        setup_vertex_args(&mut builder);
        builder.with_raw_global("camera_position");
        builder.with_raw_global("line_width");
        builder.with_primitive_type(Primitive::PointList);
        builder.with_output("color", Some(DepthMode::LessEqualWrite));

        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        mut factory: Factory,
        (active, camera, global, lines_components, lines_resource, lines_params): <Self as PassData<'a>>::Data,
    ) {
        trace!("Drawing debug lines pass");
        let debug_lines = {
            let mut lines = Vec::<DebugLine>::new();

            for debug_lines_component in (&lines_components).join() {
                lines.extend(&debug_lines_component.lines);
            }

            if let Some(mut lines_resource) = lines_resource {
                lines.append(&mut lines_resource.lines);
            };

            lines
        };

        if debug_lines.len() == 0 {
            effect.clear();
            return;
        }

        let camera = get_camera(active, &camera, &global);
        effect.update_global(
            "camera_position",
            camera
                .as_ref()
                .map(|&(_, ref trans)| trans.0.column(3).xyz().into())
                .unwrap_or([0.0; 3]),
        );

        effect.update_global("line_width", lines_params.line_width);

        let mesh = Mesh::build(debug_lines)
            .build(&mut factory)
            .expect("Failed to create debug lines mesh");

        if !set_attribute_buffers(effect, &mesh, &[V::QUERIED_ATTRIBUTES]) {
            effect.clear();
            return;
        }

        set_vertex_args(
            effect,
            encoder,
            camera,
            &GlobalTransform(na::one()),
            Rgba::WHITE,
        );

        effect.draw(mesh.slice(), encoder);
        effect.clear();
    }
}
