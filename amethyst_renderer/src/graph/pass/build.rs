use gfx_hal::{Backend, Device, Primitive};
use gfx_hal::command::{ClearColor, ClearDepthStencil, ClearValue};
use gfx_hal::device::Extent;
use gfx_hal::format::Format;
use gfx_hal::image;
use gfx_hal::pass;
use gfx_hal::pso;

use descriptors::Descriptors;
use graph::{ErrorKind, PassNode, Result, SuperFramebuffer};
use graph::pass::{AnyPass, Pass};
use shaders::ShaderManager;
use vertex::VertexFormat;


/// Collection of data required to construct rendering pass
#[derive(Derivative)]
#[derivative(Debug)]
pub struct PassBuilder<'a, B: Backend> {
    pub(crate) inputs: &'a [Format],
    pub(crate) colors: &'a [Format],
    pub(crate) depth_stencil: Option<Format>,
    pub(crate) rasterizer: pso::Rasterizer,
    pub(crate) primitive: Primitive,
    pub(crate) connects: Vec<Pin<'a, B>>,
    name: &'a str,
    #[derivative(Debug = "ignore")]
    pub(crate) maker: fn() -> Box<AnyPass<B>>,
}

#[derive(Debug)]
pub(crate) enum AttachmentImageViews<'a, B: Backend> {
    Owned(&'a [B::ImageView]),
    Acquired(&'a [B::ImageView]),
    External,
}

#[derive(Debug)]
pub(crate) struct InputAttachmentDesc<'a, B: Backend> {
    pub(crate) format: Format,
    pub(crate) view: &'a [B::ImageView],
}

#[derive(Debug)]
pub(crate) struct ColorAttachmentDesc<'a, B: Backend> {
    pub(crate) format: Format,
    pub(crate) view: AttachmentImageViews<'a, B>,
    pub(crate) clear: Option<ClearColor>,
}

#[derive(Debug)]
pub(crate) struct DepthStencilAttachmentDesc<'a, B: Backend> {
    pub(crate) format: Format,
    pub(crate) view: AttachmentImageViews<'a, B>,
    pub(crate) clear: Option<ClearDepthStencil>,
}

impl<'a, B> PassBuilder<'a, B>
where
    B: Backend,
{
    /// Construct `PassBuilder` from `Pass` type.
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
            maker: P::maker,
        }
    }

    /// Build `PassNode`
    pub(crate) fn build(
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
                let attachment = pass::Attachment {
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
                };
                println!("Init depth attachment {:?}", attachment);
                attachment
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

        let descriptors = Descriptors::new(pass.bindings(), device);

        // Create `PipelineLayout` from single `DescriptorSetLayout`
        let pipeline_layout = device.create_pipeline_layout(&[descriptors.layout()], &[]);

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
        clears.extend(
            depth_stencil
                .as_ref()
                .and_then(|ds| ds.clear)
                .map(ClearValue::DepthStencil),
        );

        let mut draws_to_surface = false;
        // create framebuffers
        let framebuffer: SuperFramebuffer<B> = {
            if colors.len() == 1 && match colors[0].view {
                AttachmentImageViews::External => true,
                _ => false,
            } {
                SuperFramebuffer::External
            } else {
                let mut frames = None;

                colors
                    .iter()
                    .map(|c| &c.view)
                    .chain(depth_stencil.iter().map(|ds| &ds.view))
                    .map(|view| match *view {
                        AttachmentImageViews::Acquired(ref images) => {
                            draws_to_surface = true;
                            images
                        }
                        AttachmentImageViews::Owned(ref images) => images,
                        AttachmentImageViews::External => {
                            unreachable!("External framebuffer isn't valid for multicolor output")
                        }
                    })
                    .for_each(|images| {
                        let frames = frames.get_or_insert_with(|| vec![vec![]; images.len()]);
                        assert_eq!(frames.len(), images.len());
                        for i in 0..images.len() {
                            frames[i].push(&images[i]);
                        }
                    });

                let frames = frames.unwrap_or(vec![]);

                if frames.len() > 1 {
                    assert!(
                        frames[1..]
                            .iter()
                            .all(|targets| targets.len() == frames[0].len())
                    );
                }

                SuperFramebuffer::Owned(frames
                    .iter()
                    .map(|targets| {
                        device
                            .create_framebuffer(&render_pass, targets, extent)
                            .map_err(|_| ErrorKind::FramebufferError.into())
                    })
                    .collect::<Result<Vec<_>>>()?)
            }
        };

        Ok(PassNode {
            clears,
            descriptors,
            pipeline_layout,
            graphics_pipeline,
            render_pass,
            framebuffer,
            pass,
            depends: None,
            draws_to_surface,
        })
    }
}


/// Merges output of several passes to the single set of targets.
#[derive(Clone, Debug)]
pub struct Merge<'a, B: Backend> {
    pub(crate) clear_color: Option<ClearColor>,
    pub(crate) clear_depth: Option<ClearDepthStencil>,
    pub(crate) passes: &'a [&'a PassBuilder<'a, B>],
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

    pub(crate) fn colors(&self) -> usize {
        self.passes[0].colors.len()
    }

    pub fn color(&self, index: usize) -> ColorPin<B> {
        ColorPin { merge: self, index }
    }

    pub fn depth(&self) -> Option<DepthPin<B>> {
        self.passes[0]
            .depth_stencil
            .map(|_| DepthPin { merge: self })
    }
}


/// Single output target of `Merge`.
/// It can be connected to the input of the `PassBuilder`
#[derive(Clone, Debug)]
pub struct ColorPin<'a, B: Backend> {
    pub(crate) merge: &'a Merge<'a, B>,
    pub(crate) index: usize,
}

impl<'a, B> ColorPin<'a, B>
where
    B: Backend,
{
    pub fn format(&self) -> Format {
        self.merge.passes[0].colors[self.index]
    }
}

/// Depth output target of the `Merge`
/// It can be connected to the input of the `PassBuilder`
#[derive(Clone, Debug)]
pub struct DepthPin<'a, B: Backend> {
    pub(crate) merge: &'a Merge<'a, B>,
}

impl<'a, B> DepthPin<'a, B>
where
    B: Backend,
{
    pub fn format(&self) -> Format {
        self.merge.passes[0].depth_stencil.unwrap()
    }
}

/// Common connection pin.
/// `ColorPin` or `DepthPin`.
#[derive(Clone, Debug)]
pub enum Pin<'a, B: Backend> {
    Color(ColorPin<'a, B>),
    Depth(DepthPin<'a, B>),
}

impl<'a, B> Pin<'a, B>
where
    B: Backend,
{
    fn merge(&self) -> &Merge<'a, B> {
        match *self {
            Pin::Color(ref color) => &color.merge,
            Pin::Depth(ref depth) => &depth.merge,
        }
    }

    pub fn format(&self) -> Format {
        match *self {
            Pin::Color(ref color) => color.format(),
            Pin::Depth(ref depth) => depth.format(),
        }
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
        .map(|a| a.location + 1)
        .unwrap_or(0);
    for attribute in format.attributes.iter() {
        pipeline_desc.attributes.push(pso::AttributeDesc {
            location,
            binding: index,
            element: attribute.2,
        });
        location += 1;
    }
}

/// Searches items from `right` in `left`
/// Returns indices of them.
/// Returns `None` if at least one item in `right` is not found.
pub fn indices_in_of<T>(left: &[&T], right: &[&T]) -> Option<Vec<usize>> {
    let mut positions = right
        .iter()
        .map(|&r| {
            left.iter().rposition(|&l| l as *const _ == r as *const _)
        })
        .collect::<Option<Vec<_>>>()?;
    positions.sort();
    Some(positions)
}

/// Searches items from `right` in `left`
/// Returns found indices of them.
pub fn some_indices_in_of<T>(left: &[&T], right: &[&T]) -> Vec<usize> {
    let mut positions = right
        .iter()
        .filter_map(|&r| {
            left.iter().rposition(|&l| l as *const _ == r as *const _)
        })
        .collect::<Vec<_>>();
    positions.sort();
    positions
}

fn walk_dependencies<'a, B>(pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    use std::iter::once;
    pass.connects
        .iter()
        .flat_map(|pin| {
            pin.merge()
                .passes
                .iter()
                .flat_map(|&pass| once(pass).chain(walk_dependencies(pass)))
        })
        .collect()
}

/// Get all dependencies of pass.
pub fn dependencies<'a, B>(pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut deps = walk_dependencies(pass);
    deps.sort_by_key(|p| p as *const _);
    deps.dedup_by_key(|p| p as *const _);
    deps
}

/// Get dependencies of pass that aren't dependency of dependency.
pub fn direct_dependencies<'a, B>(pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut alldeps = dependencies(pass);
    let mut newdeps = vec![];
    while let Some(dep) = alldeps.pop() {
        newdeps.push(dep);
        let other = dependencies(dep);
        alldeps.retain(|dep| indices_in_of(&other, &[dep]).is_none());
        newdeps.retain(|dep| indices_in_of(&other, &[dep]).is_none());
    }
    newdeps
}


/// Walk from pin over merges and dependencies.
/// And collect all passes
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
            pass.connects
                .iter()
                .map(|pin| pin.merge())
                .flat_map(walk_merges)
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
