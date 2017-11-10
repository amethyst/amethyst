
use gfx_hal::{Backend, Device, Primitive};
use gfx_hal::command::{ClearColor, ClearDepthStencil, ClearValue, ColorValue};
use gfx_hal::format::Format;
use gfx_hal::pso;
use gfx_hal::pass;
use gfx_hal::image;

use specs::{Component, Entity, World};


use graph::pass::{AnyPass, Pass};
use graph::{Error, ErrorKind, PassNode, Result};
use vertex::VertexFormat;
use uniform::IntoUniform;

pub struct PassBuilder<'a, B: Backend> {
    inputs: &'a [Format],
    colors: &'a [Format],
    depth_stencil: Option<Format>,
    bindings: &'a [pso::DescriptorSetLayoutBinding],
    vertices: &'a [VertexFormat<'a>],

    shaders: pso::GraphicsShaderSet<'a, B>,
    rasterizer: pso::Rasterizer,

    primitive: Primitive,

    connect: Vec<(&'a PassBuilder<'a, B>, usize)>,
    pass: Box<AnyPass<B>>,
}

#[derive(Debug)]
pub enum AttachmentImageView<'a, B: Backend> {
    Owned(&'a B::ImageView),
    Acquired(&'a [B::ImageView]),
    Single,
}

#[derive(Debug)]
pub struct InputAttachmentDesc<'a, B: Backend> {
    format: Format,
    view: AttachmentImageView<'a, B>,
}

#[derive(Debug)]
pub struct ColorAttachmentDesc<'a, B: Backend> {
    format: Format,
    view: AttachmentImageView<'a, B>,
    clear: Option<ClearColor>,
}

#[derive(Debug)]
pub struct DepthStencilAttachmentDesc<'a, B: Backend> {
    format: Format,
    view: AttachmentImageView<'a, B>,
    clear: Option<ClearDepthStencil>,
}

impl<'a, B> PassBuilder<'a, B>
where
    B: Backend,
{
    pub fn build(
        self,
        device: &mut B::Device,
        inputs: &[InputAttachmentDesc<B>],
        colors: &[ColorAttachmentDesc<B>],
        depth_stencil: Option<DepthStencilAttachmentDesc<B>>,
    ) -> Result<PassNode<B>> {

        // This is enforced by `RenderGraphBuilder`
        assert_eq!(inputs.len(), self.inputs.len());
        assert_eq!(colors.len(), self.colors.len());
        assert_eq!(depth_stencil.is_some(), self.depth_stencil.is_some());

        // Construct `RenderPass`
        // with single `Subpass` for now
        let render_pass = {
            // Configure input attachments first
            let inputs = self.inputs.iter().cloned().enumerate().map(|(i, format)| {
                assert_eq!(inputs[i].format, format);
                pass::Attachment {
                    format,
                    ops: pass::AttachmentOps {
                        load: pass::AttachmentLoadOp::Load,
                        store: pass::AttachmentStoreOp::Store,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: image::ImageLayout::General..image::ImageLayout::General,
                }
            });

            // Configure color attachments next to input
            let colors = self.colors.iter().cloned().enumerate().map(|(i, format)| {
                assert_eq!(colors[i].format, format);
                pass::Attachment {
                    format,
                    ops: pass::AttachmentOps {
                        load: if colors[i].clear.is_some() {
                            pass::AttachmentLoadOp::Clear
                        } else {
                            pass::AttachmentLoadOp::Load
                        },
                        store: pass::AttachmentStoreOp::Store,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: if colors[i].clear.is_some() {
                        image::ImageLayout::Undefined
                    } else {
                        image::ImageLayout::General
                    }..image::ImageLayout::General,
                }
            });

            // Configure depth-stencil attachments last
            let depth_stencil = self.depth_stencil.clone().map(|format| {
                let depth_stencil = depth_stencil.as_ref().unwrap();
                assert_eq!(depth_stencil.format, format);
                pass::Attachment {
                    format,
                    ops: pass::AttachmentOps {
                        load: if depth_stencil.clear.is_some() {
                            pass::AttachmentLoadOp::Clear
                        } else {
                            pass::AttachmentLoadOp::Load
                        },
                        store: pass::AttachmentStoreOp::Store,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: if depth_stencil.clear.is_some() {
                        image::ImageLayout::Undefined
                    } else {
                        image::ImageLayout::General
                    }..image::ImageLayout::General,
                }
            });

            let depth_stencil_ref = depth_stencil.as_ref().map(|_| {
                (inputs.len() + colors.len(), image::ImageLayout::General)
            });

            // Configure the only `Subpass` using all attachments
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

            // Create `RenderPass` with all attachments
            // and single `Subpass`
            device.create_render_pass(
                &inputs
                    .chain(colors)
                    .chain(depth_stencil)
                    .collect::<Vec<_>>(),
                &[subpass],
                &[], // TODO: Add external subpass dependency
            )
        };

        // Create `DescriptorSetLayout` from bindings
        let descriptor_set_layout = device.create_descriptor_set_layout(self.bindings);

        // Create `PipelineLayout` from single `DescriptorSetLayout`
        let pipeline_layout = device.create_pipeline_layout(&[&descriptor_set_layout]);

        // Create `GraphicsPipeline`
        let pipeline = {
            // Init basic configuration
            let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                self.shaders,
                self.primitive,
                self.rasterizer,
                &pipeline_layout,
                pass::Subpass {
                    index: 0,
                    main_pass: &render_pass,
                },
            );

            // Default configuration for blending targets for all color targets
            pipeline_desc.blender.targets =
                vec![
                    pso::ColorBlendDesc(pso::ColorMask::ALL, pso::BlendState::ALPHA);
                    self.colors.len()
                ];

            // Default configuration for depth-stencil
            pipeline_desc.depth_stencil = Some(pso::DepthStencilDesc {
                depth: pso::DepthTest::On {
                    fun: pso::Comparison::LessEqual,
                    write: true,
                },
                depth_bounds: false,
                stencil: pso::StencilTest::Off,
            });

            // Add all vertex descriptors
            for vertex in self.vertices {
                push_vertex_desc(vertex, &mut pipeline_desc);
            }

            // Create `GraphicsPipeline`
            device
                .create_graphics_pipelines(&[pipeline_desc])
                .pop()
                .unwrap()?
        };

        // This color will be set to targets that aren't get cleared
        let ignored_color = ClearColor::Float([0.1, 0.2, 0.3, 1.0]);

        // But we need `ClearValue` for each target
        let mut clears = vec![ClearValue::Color(ignored_color); inputs.len()];

        // Add those for colors
        clears.extend(
            colors
                .iter()
                .map(|c| c.clear.unwrap_or(ignored_color))
                .map(ClearValue::Color),
        );

        // And depth-stencil
        clears.extend(depth_stencil.and_then(|ds| ds.clear).map(
            ClearValue::DepthStencil,
        ));

        // TODO: Construct `PassNode`
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
