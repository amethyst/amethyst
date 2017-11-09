
use gfx_hal::{Backend, Device, Primitive};
use gfx_hal::format::Format;
use gfx_hal::pso;
use gfx_hal::pass;
use gfx_hal::image;

use specs::{Component, Entity, World};


use graph::pass::{AnyPass, Pass};
use graph::{Error, ErrorKind, PassNode, Result};
use vertex::VertexFormat;
use uniform::IntoUniform;

pub struct PassDesc<'a, B: Backend> {
    inputs: &'a [Format],
    colors: &'a [Format],
    depth_stencil: Option<Format>,
    bindings: &'a [pso::DescriptorSetLayoutBinding],
    vertices: &'a [VertexFormat<'a>],

    shaders: pso::GraphicsShaderSet<'a, B>,
    rasterizer: pso::Rasterizer,

    primitive: Primitive,

    connect: Vec<(&'a PassDesc<'a, B>, usize)>,
    pass: Box<AnyPass<B>>,
}

impl<'a, B> PassDesc<'a, B>
where
    B: Backend,
{
    fn build(
        self,
        device: &mut B::Device,
        colors_clear: &[bool],
        depth_clear: bool,
    ) -> Result<PassNode<B>> {

        let render_pass = {
            let inputs = self.inputs.iter().cloned().map(|format| {
                pass::Attachment {
                    format,
                    ops: pass::AttachmentOps {
                        load: pass::AttachmentLoadOp::Load,
                        store: pass::AttachmentStoreOp::DontCare,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: image::ImageLayout::General..image::ImageLayout::General,
                }
            });

            let colors = self.colors.iter().cloned().enumerate().map(|(i, format)| {
                pass::Attachment {
                    format,
                    ops: pass::AttachmentOps {
                        load: if colors_clear[i] {
                            pass::AttachmentLoadOp::Clear
                        } else {
                            pass::AttachmentLoadOp::Load
                        },
                        store: pass::AttachmentStoreOp::Store,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: if colors_clear[i] {
                        image::ImageLayout::Undefined
                    } else {
                        image::ImageLayout::General
                    }..image::ImageLayout::General,
                }
            });

            let depth_stencil = self.depth_stencil.clone().map(|format| {
                pass::Attachment {
                    format,
                    ops: pass::AttachmentOps {
                        load: if depth_clear {
                            pass::AttachmentLoadOp::Clear
                        } else {
                            pass::AttachmentLoadOp::Load
                        },
                        store: pass::AttachmentStoreOp::Store,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: if depth_clear {
                        image::ImageLayout::Undefined
                    } else {
                        image::ImageLayout::General
                    }..image::ImageLayout::General,
                }
            });

            let depth_stencil_ref = depth_stencil.as_ref().map(|_| {
                (inputs.len() + colors.len(), image::ImageLayout::General)
            });

            let subpass = pass::SubpassDesc {
                colors: &(0..colors.len())
                    .map(|i| (i + inputs.len(), image::ImageLayout::General))
                    .collect::<Vec<_>>(),
                depth_stencil: depth_stencil_ref.as_ref(),
                inputs: &(0..inputs.len())
                    .map(|i| (i, image::ImageLayout::General))
                    .collect::<Vec<_>>(),
                preserves: &[],
            };

            device.create_render_pass(
                &inputs
                    .chain(colors)
                    .chain(depth_stencil)
                    .collect::<Vec<_>>(),
                &[subpass],
                &[],
            )
        };

        let descriptor_set_layout = device.create_descriptor_set_layout(self.bindings);
        let pipeline_layout = device.create_pipeline_layout(&[&descriptor_set_layout]);

        let pipeline = {
            let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                self.shaders,
                self.primitive,
                self.rasterizer,
                &pipeline_layout,
                pass::Subpass {
                    index: 1,
                    main_pass: &render_pass,
                },
            );
            pipeline_desc.blender.targets =
                vec![
                    pso::ColorBlendDesc(pso::ColorMask::ALL, pso::BlendState::ALPHA);
                    self.colors.len()
                ];
            pipeline_desc.depth_stencil = Some(pso::DepthStencilDesc {
                depth: pso::DepthTest::On {
                    fun: pso::Comparison::GreaterEqual,
                    write: true,
                },
                depth_bounds: false,
                stencil: pso::StencilTest::Off,
            });

            for vertex in self.vertices {
                push_vertex_desc(vertex, &mut pipeline_desc);
            }

            device
                .create_graphics_pipelines(&[pipeline_desc])
                .pop()
                .unwrap()?
        };

        unimplemented!()
    }
}


fn push_vertex_desc<B>(format: &VertexFormat, pipeline_desc: &mut pso::GraphicsPipelineDesc<B>)
where
    B: Backend,
{
    let index = pipeline_desc.vertex_buffers.len() as pso::BufferIndex;

    pipeline_desc.vertex_buffers.push(pso::VertexBufferDesc {
        stride: format.stride,
        rate: 0,
    });

    let mut location = 0;
    for attribute in format.attributes.iter() {
        pipeline_desc.attributes.push(pso::AttributeDesc {
            location,
            binding: index,
            element: attribute.1,
        });
        location += 1;
    }
}
