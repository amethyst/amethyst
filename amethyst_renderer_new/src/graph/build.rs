use std::collections::HashSet;

use gfx_hal::{Backend, Device, Primitive};
use gfx_hal::command::{ClearColor, ClearDepthStencil, ClearValue, ColorValue};
use gfx_hal::device::Extent;
use gfx_hal::format::Format;
use gfx_hal::image;
use gfx_hal::pass;
use gfx_hal::pso;

use specs::{Component, Entity, World};

use graph::{Error, ErrorKind, PassNode, Result, SuperFramebuffer};
use graph::pass::{AnyPass, Pass};
use shaders::ShaderManager;
use vertex::VertexFormat;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PassBuilder<'a, B: Backend> {
    pub(super) inputs: &'a [Format],
    pub(super) colors: &'a [Format],
    pub(super) depth_stencil: Option<Format>,
    pub(super) rasterizer: pso::Rasterizer,
    pub(super) primitive: Primitive,
    pub(super) connects: Vec<Pin<'a, B>>,
    name: &'a str,
    #[derivative(Debug = "ignore")]
    pub(super) maker: Box<Fn() -> Box<AnyPass<B>>>,
}

#[derive(Debug)]
pub enum AttachmentImageView<'a, B: Backend> {
    Owned(&'a B::ImageView),
    Acquired(&'a [B::ImageView]),
    Single,
}

#[derive(Debug)]
pub struct InputAttachmentDesc<'a, B: Backend> {
    pub format: Format,
    pub view: &'a B::ImageView,
}

#[derive(Debug)]
pub struct ColorAttachmentDesc<'a, B: Backend> {
    pub format: Format,
    pub view: AttachmentImageView<'a, B>,
    pub clear: Option<ClearColor>,
}

#[derive(Debug)]
pub struct DepthStencilAttachmentDesc<'a, B: Backend> {
    pub format: Format,
    pub view: AttachmentImageView<'a, B>,
    pub clear: Option<ClearDepthStencil>,
}

impl<'a, B> PassBuilder<'a, B>
where
    B: Backend,
{
    pub fn new<P>() -> Self
    where
        P: Pass<B> + 'static,
    {
        PassBuilder {
            inputs: P::INPUTS,
            colors: P::COLORS,
            depth_stencil: P::DEPTH_STENCIL,
            rasterizer: pso::Rasterizer::FILL,
            primitive: Primitive::TriangleList,
            connects: vec![],
            name: P::NAME,
            maker: P::maker(),
        }
    }

    pub fn build(
        &self,
        device: &B::Device,
        shaders: &mut ShaderManager<B>,
        inputs: &[InputAttachmentDesc<B>],
        colors: &[ColorAttachmentDesc<B>],
        depth_stencil: Option<DepthStencilAttachmentDesc<B>>,
        extent: Extent,
    ) -> Result<PassNode<B>> {
        let pass = (self.maker)();
        /// Check connects
        assert_eq!(pass.inputs().len(), self.connects.len());
        for (i, pin) in self.connects.iter().enumerate() {
            assert_eq!(pin.format(), pass.inputs()[i]);
        }

        // Check attachments
        assert_eq!(inputs.len(), pass.inputs().len());
        assert_eq!(colors.len(), pass.colors().len());
        assert_eq!(depth_stencil.is_some(), pass.depth_stencil().is_some());

        // Construct `RenderPass`
        // with single `Subpass` for now
        let render_pass = {
            // Configure input attachments first
            let inputs = pass.inputs().iter().enumerate().map(|(i, &format)| {
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
            let colors = pass.colors().iter().enumerate().map(|(i, &format)| {
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
            let depth_stencil = pass.depth_stencil().clone().map(|format| {
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
                (
                    inputs.len() + colors.len(),
                    image::ImageLayout::DepthStencilAttachmentOptimal,
                )
            });

            // Configure the only `Subpass` using all attachments
            let subpass = pass::SubpassDesc {
                colors: &(0..colors.len())
                    .map(|i| {
                        (i + inputs.len(), image::ImageLayout::ColorAttachmentOptimal)
                    })
                    .collect::<Vec<_>>(),
                depth_stencil: depth_stencil_ref.as_ref(),
                inputs: &(0..inputs.len())
                    .map(|i| (i, image::ImageLayout::ShaderReadOnlyOptimal))
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
        let descriptor_set_layout = device.create_descriptor_set_layout(pass.bindings());

        // Create `PipelineLayout` from single `DescriptorSetLayout`
        let pipeline_layout = device.create_pipeline_layout(&[&descriptor_set_layout], &[]);

        // Create `GraphicsPipeline`
        let graphics_pipeline = {
            // Init basic configuration
            let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                pass.shaders(shaders, device)?,
                self.primitive,
                self.rasterizer.clone(),
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
                    pass.colors().len()
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
            for vertex in pass.vertices() {
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
        clears.extend(depth_stencil.as_ref().and_then(|ds| ds.clear).map(
            ClearValue::DepthStencil,
        ));

        // create framebuffers
        let framebuffer: SuperFramebuffer<B> = {
            if colors.len() == 1 &&
                match colors[0].view {
                    AttachmentImageView::Single => true,
                    _ => false,
                }
            {
                SuperFramebuffer::Single
            } else {
                let mut acquired = None;
                let mut targets = colors
                    .iter()
                    .map(|c| &c.view)
                    .chain(depth_stencil.iter().map(|ds| &ds.view))
                    .enumerate()
                    .map(|(index, view)| match *view {
                        AttachmentImageView::Owned(ref image) => image,
                        AttachmentImageView::Acquired(ref images) => {
                            match acquired {
                                Some(_) => unreachable!("Only one acquried target"),
                                ref mut acquired @ None => *acquired = Some((index, images)),
                            }
                            &images[0]
                        }
                        AttachmentImageView::Single => {
                            unreachable!("Single framebuffer isn't valid for multicolor output")
                        }
                    })
                    .collect::<Vec<_>>();

                if let Some((index, images)) = acquired {
                    SuperFramebuffer::Acquired(images
                        .iter()
                        .map(|image| {
                            targets[index] = image;
                            device
                                .create_framebuffer(&render_pass, &targets[..], extent)
                                .map_err(|_| ErrorKind::FramebufferError.into())
                        })
                        .collect::<Result<Vec<_>>>()?)
                } else {
                    SuperFramebuffer::Owned(device
                        .create_framebuffer(&render_pass, &targets[..], extent)
                        .map_err(|_| ErrorKind::FramebufferError)?)
                }
            }
        };

        Ok(PassNode {
            clears,
            descriptor_set_layout,
            pipeline_layout,
            graphics_pipeline,
            render_pass,
            framebuffer,
            pass,
            depends: vec![],
        })
    }
}



#[derive(Clone, Debug)]
pub struct Merge<'a, B: Backend> {
    pub(super) clear_color: Option<ClearColor>,
    pub(super) clear_depth: Option<ClearDepthStencil>,
    pub(super) passes: &'a [&'a PassBuilder<'a, B>],
}

impl<'a, B> Merge<'a, B>
where
    B: Backend,
{
    pub fn new(
        clear_color: Option<ClearColor>,
        clear_depth: Option<ClearDepthStencil>,
        passes: &'a [&'a PassBuilder<'a, B>],
    ) -> Self {
        assert!(!passes.is_empty());
        let colors = passes[0].colors;
        let depth_stencil = passes[0].depth_stencil;
        for pass in &passes[1..] {
            assert_eq!(colors, pass.colors);
            assert_eq!(depth_stencil, pass.depth_stencil);
        }
        Merge {
            clear_color,
            clear_depth,
            passes,
        }
    }

    pub fn colors(&self) -> usize {
        self.passes[0].colors.len()
    }

    pub fn color_format(&self, index: usize) -> Format {
        self.passes[0].colors[index]
    }

    pub fn depth_format(&self) -> Option<Format> {
        self.passes[0].depth_stencil
    }
}

#[derive(Clone, Debug)]
pub struct ColorPin<'a, B: Backend> {
    pub(super) merge: &'a Merge<'a, B>,
    pub(super) index: usize,
}

impl<'a, B> ColorPin<'a, B>
where
    B: Backend,
{
    pub fn new(merge: &'a Merge<'a, B>, index: usize) -> Self {
        assert!(merge.colors() > index);
        ColorPin { merge, index }
    }
    pub fn format(&self) -> Format {
        self.merge.color_format(self.index)
    }
}

#[derive(Clone, Debug)]
pub struct DepthPin<'a, B: Backend> {
    pub(super) merge: &'a Merge<'a, B>,
}

impl<'a, B> DepthPin<'a, B>
where
    B: Backend,
{
    fn format(&self) -> Format {
        self.merge.depth_format().unwrap()
    }
}

#[derive(Clone, Debug)]
pub enum Pin<'a, B: Backend> {
    Color(ColorPin<'a, B>),
    Depth(DepthPin<'a, B>),
}

impl<'a, B> Pin<'a, B>
where
    B: Backend,
{
    fn format(&self) -> Format {
        match *self {
            Pin::Color(ref color) => color.format(),
            Pin::Depth(ref depth) => depth.format(),
        }
    }

    fn merge(&'a self) -> &'a Merge<'a, B> {
        match *self {
            Pin::Color(ref color) => &color.merge,
            Pin::Depth(ref depth) => &depth.merge,
        }
    }
}


///
#[derive(Debug)]
pub struct Present<'a, B: Backend> {
    pub(super) pin: ColorPin<'a, B>,
}

impl<'a, B> Present<'a, B>
where
    B: Backend,
{
    pub fn new(pin: ColorPin<'a, B>) -> Self {
        Present { pin }
    }
    pub fn format(&self) -> Format {
        self.pin.format()
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

    let mut location = pipeline_desc
        .attributes
        .last()
        .map(|a| a.location)
        .unwrap_or(0);
    for attribute in format.attributes.iter() {
        pipeline_desc.attributes.push(pso::AttributeDesc {
            location,
            binding: attribute.0,
            element: attribute.2,
        });
        location += 1;
    }
}

pub fn dependency_search<T>(left: &[&T], right: &[&T]) -> Option<Vec<usize>> {
    use std::result::Result as StdResult;

    fn _search<T>(left: &[&T], right: &[&T]) -> StdResult<Vec<usize>, ()> {
        let mut positions = right
            .iter()
            .map(|&r| {
                left.iter()
                    .rposition(|&l| l as *const _ == r as *const _)
                    .ok_or(())
            })
            .collect::<StdResult<Vec<_>, _>>()?;
        positions.sort();
        Ok(positions)
    };
    _search(left, right).ok()
}

fn walk_dependencies<'a, B>(pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    use std::iter::once;
    pass.connects
        .iter()
        .flat_map(|pin| {
            pin.merge().passes.iter().flat_map(|&pass| {
                once(pass).chain(walk_dependencies(pass))
            })
        })
        .collect()
}

pub fn dependencies<'a, B>(pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut deps = walk_dependencies(pass);
    deps.sort_by_key(|p| p as *const _);
    deps.dedup_by_key(|p| p as *const _);
    deps
}

pub fn direct_dependencies<'a, B>(pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut alldeps = dependencies(pass);
    let mut newdeps = vec![];
    while let Some(dep) = alldeps.pop() {
        newdeps.push(dep);
        let other = dependencies(dep);
        alldeps.retain(|dep| dependency_search(&other, &[dep]).is_none());
        newdeps.retain(|dep| dependency_search(&other, &[dep]).is_none());
    }
    newdeps
}

pub fn traverse<'a, B>(pin: &'a ColorPin<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut passes = pin.merge
        .passes
        .iter()
        .flat_map(|&pass| {
            let mut passes = walk_dependencies(pass);
            passes.push(pass);
            passes
        })
        .collect::<Vec<_>>();
    passes.sort_by_key(|p| p as *const _);
    passes.dedup_by_key(|p| p as *const _);
    passes
}

fn walk_merges<'a, B>(merge: &'a Merge<'a, B>) -> Vec<&'a Merge<'a, B>>
where
    B: Backend,
{
    use std::iter::once;
    once(merge)
        .chain(merge.passes.iter().flat_map(|&pass| {
            pass.connects.iter().map(|pin| pin.merge()).flat_map(
                |merge| {
                    walk_merges(merge)
                },
            )
        }))
        .collect()
}

pub fn merges<'a, B>(pin: &'a ColorPin<'a, B>) -> Vec<&'a Merge<'a, B>>
where
    B: Backend,
{
    let mut merges = walk_merges(pin.merge);
    merges.sort_by_key(|p| p as *const _);
    merges.dedup_by_key(|p| p as *const _);
    merges
}
