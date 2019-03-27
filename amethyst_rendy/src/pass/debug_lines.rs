//! Debug lines pass

use std::marker::PhantomData;

use derivative::Derivative;
use gfx::{pso::buffer::ElemStride, Primitive};
use log::{debug, trace};

use amethyst_core::{
    ecs::{Join, Read, ReadStorage, Write, WriteStorage},
    math as na,
    transform::GlobalTransform,
};
use amethyst_error::Error;

use crate::{
    cam::{ActiveCamera, Camera},
    debug_drawing::{DebugLine, DebugLines, DebugLinesComponent},
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

pub struct DrawDebugLinesDesc {
    pub line_width: f32
}

impl Default for DrawDebugLinesDesc {
    fn default() -> Self {
        DrawDebugLines {
            line_width: 1.0 / 400.0
        }
    }
}

impl DrawDebugLinesDesc {
    pub fn new() -> Self {
        Default::default()
    }    
}

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawDebugLinesDesc {
    type Pipeline = DrawDebugLines<B>;

    fn load_shader_set<'a>(
            &self,
            storage: &'a mut Vec<B::ShaderModule>,
            factory: &mut Factory<B>,
            _aux: &Resources,
        ) -> GraphicsShaderSet<'a, B> {
            storage.clear();

            log::trace!("Loading shader module '{:#?}'", *super::DEBUG_LINES_VERTEX);
            storage.push(super::DEBUG_LINES_VERTEX.module(factory).unwrap());

            log::trace!("Loading shader module '{:#?}'", *super::DEBUG_LINES_FLAT);
            storage.push(super::DEBUG_LINES_FLAT.module(factory).unwrap());

            log::trace!("Loading shader module '{:#?}'", *super::DEBUG_LINES_GEOM);
            storage.push(super::DEBUG_LINES_GEOM.module(factory).unwrap());

            GraphicsShaderSet {
                vertex: EntryPoint {
                    entry: "main",
                    module: &storage[0],
                    specialization: Specialization::default(),
                },
                fragment: Some(EntryPoint {
                    entry: "main",
                    module: &storage[1],
                    specialization: Specialization::default(),
                }),
                fragment: Some(EntryPoint {
                    entry: "main",
                    module: &storage[2],
                    specialization: Specialization::default(),
                }),
                hull: None,
                domain: None,
            }
    }

    fn colors(&self) -> Vec<ColorBlendDesc> {
        vec![ColorMask::WHITE]
    }

    fn build<'a>(
        self,
        _ctx: &mut GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _resources: &Resources,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        _set_layouts: &[DescriptorSetLayout<B>],
    ) -> Result<DrawDebugLines<B>, failure::Error> {
        let buffer = factory.create_buffer(1, 1024, rendy::resource::buffer::UniformBuffer)?;
        Ok(DrawDebugLines {
            buffer,
        })
    }
}

#[derive(Debug)]
pub struct DrawDebugLines<B: Backend> {
    buffer: rendy::resource::Buffer<B>
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawDebugLines<B> {
    type Desc = DrawDebugLinesDesc;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[DescriptorSetLayout<B>],
        _index: usize,
        resources: &Resources,
    ) -> PrepareResult {
 let (
    active_camera,
    cameras,
    mesh_storage,
    texture_storage,
    material_defaults,
    visibility,
    hiddens,
    meshes,
    materials,
    globals,
    joints,
    ) = <(
    Option<Read<'_, ActiveCamera>>,
    ReadStorage<'_, Camera>,
    Read<'_, AssetStorage<Mesh<B>>>,
    Read<'_, AssetStorage<Texture<B>>>,
    ReadExpect<'_, MaterialDefaults<B>>,
    Option<Read<'_, Visibility>>,
    ReadStorage<'_, Hidden>,
    ReadStorage<'_, Handle<Mesh<B>>>,
    ReadStorage<'_, Handle<Material<B>>>,
    ReadStorage<'_, GlobalTransform>,
    ReadStorage<'_, JointTransforms>,
    ) as SystemData>::fetch(resources);

    let defcam = Camera::standard_2d();
    let identity = GlobalTransform::default();
    let camera = active_camera
        .and_then(|ac| {
            cameras
                .get(ac.entity)
                .map(|camera| (camera, globals.get(ac.entity).unwrap_or(&identity)))
        })
        .unwrap_or_else(|| {
            (&cameras, &globals)
                .join()
                .next()
                .unwrap_or((&defcam, &identity))
        });
    }

    fn draw(
        &mut self,
        _layout: &B::PipelineLayout,
        _encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &Resources,
    ) {

    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {
        unimplemented!()
    }
}

// impl<V> Pass for DrawDebugLines<V>
// where
//     V: Query<(Position, Color, Normal)>,
// {
//     fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect, Error> {
//         debug!("Building debug lines pass");
//         let mut builder = effect.geom(VERT_SRC, GEOM_SRC, FRAG_SRC);

//         debug!("Effect compiled, adding vertex/uniform buffers");
//         builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);

//         setup_vertex_args(&mut builder);
//         builder.with_raw_global("camera_position");
//         builder.with_raw_global("line_width");
//         builder.with_primitive_type(Primitive::PointList);
//         builder.with_output("color", Some(DepthMode::LessEqualWrite));

//         builder.build()
//     }

//     fn apply<'a, 'b: 'a>(
//         &'a mut self,
//         encoder: &mut Encoder,
//         effect: &mut Effect,
//         mut factory: Factory,
//         (active, camera, global, lines_components, lines_resource, lines_params): <Self as PassData<'a>>::Data,
//     ) {
//         trace!("Drawing debug lines pass");
//         let debug_lines = {
//             let mut lines = Vec::<DebugLine>::new();

//             for debug_lines_component in (&lines_components).join() {
//                 lines.extend(&debug_lines_component.lines);
//             }

//             if let Some(mut lines_resource) = lines_resource {
//                 lines.append(&mut lines_resource.lines);
//             };

//             lines
//         };

//         if debug_lines.len() == 0 {
//             effect.clear();
//             return;
//         }

//         let camera = get_camera(active, &camera, &global);
//         effect.update_global(
//             "camera_position",
//             camera
//                 .as_ref()
//                 .map(|&(_, ref trans)| trans.0.column(3).xyz().into())
//                 .unwrap_or([0.0; 3]),
//         );

//         effect.update_global("line_width", lines_params.line_width);

//         let mesh = Mesh::build(debug_lines)
//             .build(&mut factory)
//             .expect("Failed to create debug lines mesh");

//         if !set_attribute_buffers(effect, &mesh, &[V::QUERIED_ATTRIBUTES]) {
//             effect.clear();
//             return;
//         }

//         set_vertex_args(
//             effect,
//             encoder,
//             camera,
//             &GlobalTransform(na::one()),
//             Rgba::WHITE,
//         );

//         effect.draw(mesh.slice(), encoder);
//         effect.clear();
//     }
// }
