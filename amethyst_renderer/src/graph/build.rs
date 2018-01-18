use std::ops::Range;

use gfx_hal::{Backend, Device, Primitive};
use gfx_hal::command::{ClearColor, ClearDepthStencil, ClearValue};
use gfx_hal::device::Extent;
use gfx_hal::format::{AspectFlags, Format, Swizzle};
use gfx_hal::image;
use gfx_hal::memory::Properties;
use gfx_hal::pass;
use gfx_hal::pso;

use smallvec::SmallVec;
use specs::World;

use descriptors::DescriptorPool;
use graph::{ErrorKind, PassNode, Result, SuperFramebuffer};
use graph::pass::{AnyPass, Pass};
use memory::{Allocator, Image};
use vertex::VertexFormat;

pub const COLOR_RANGE: image::SubresourceRange = image::SubresourceRange {
    aspects: AspectFlags::COLOR,
    levels: 0..1,
    layers: 0..1,
};


/// 
#[derive(Debug)]
pub struct ColorAttachment {
    pub format: Format,
    pub clear: Option<ClearColor>,
}

impl ColorAttachment {
    pub fn new(format: Format) -> Self {
        ColorAttachment {
            format,
            clear: None,
        }
    }

    pub fn with_clear(mut self, clear: ClearColor) -> Self {
        self.set_clear(clear);
        self
    }

    pub fn set_clear(&mut self, clear: ClearColor) {
        self.clear = Some(clear);
    }

    pub fn clear(mut self, clear: ClearColor) -> Self {
        self.set_clear(clear);
        self
    }

    pub(crate) fn is(&self, other: &ColorAttachment) -> bool {
        self.key() == other.key()
    }

    pub(crate) fn key(&self) -> *const () {
        self as *const _ as *const ()
    }
}

/// 
#[derive(Debug)]
pub struct DepthStencilAttachment {
    pub format: Format,
    pub clear: Option<ClearDepthStencil>,
}

impl DepthStencilAttachment {
    pub fn new(format: Format) -> Self {
        DepthStencilAttachment {
            format,
            clear: None,
        }
    }

    pub fn set_clear(&mut self, clear: ClearDepthStencil) {
        self.clear = Some(clear);
    }

    pub fn clear(mut self, clear: ClearDepthStencil) -> Self {
        self.set_clear(clear);
        self
    }

    pub(crate) fn is(&self, other: &DepthStencilAttachment) -> bool {
        self.key() == other.key()
    }

    pub(crate) fn key(&self) -> *const () {
        self as *const _ as *const ()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Attachment<'a> {
    Color(&'a ColorAttachment),
    DepthStencil(&'a DepthStencilAttachment),
}

impl<'a> Attachment<'a> {
    pub fn format(&self) -> Format {
        match *self {
            Attachment::Color(color) => color.format,
            Attachment::DepthStencil(depth) => depth.format,
        }
    }

    pub(crate) fn key(&self) -> *const () {
        match *self {
            Attachment::Color(color) => color.key(),
            Attachment::DepthStencil(depth) => depth.key(),
        }
    }

    pub(crate) fn is_color(&self, color: &ColorAttachment) -> bool {
        self.key() == color.key()
    }

    pub(crate) fn is_depth(&self, depth_stencil: &DepthStencilAttachment) -> bool {
        self.key() == depth_stencil.key()
    }
}

#[derive(Debug)]
pub(crate) enum AttachmentImageViews<'a, B: Backend> {
    Owned(&'a [B::ImageView]),
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

/// Collection of data required to construct rendering pass
#[derive(Derivative)]
#[derivative(Debug)]
pub struct PassBuilder<'a, B: Backend> {
    pub inputs: Vec<Option<Attachment<'a>>>,
    pub colors: Vec<Option<&'a ColorAttachment>>,
    pub depth_stencil: Option<(Option<&'a DepthStencilAttachment>, bool)>,
    pub rasterizer: pso::Rasterizer,
    pub primitive: Primitive,
    name: &'a str,
    #[derivative(Debug = "ignore")]
    pub(crate) maker: fn() -> Box<AnyPass<B>>,
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
            inputs: vec![None; P::INPUTS],
            colors: vec![None; P::COLORS],
            depth_stencil: if P::DEPTH { Some((None, P::STENCIL)) } else { None },
            rasterizer: pso::Rasterizer::FILL,
            primitive: Primitive::TriangleList,
            name: P::NAME,
            maker: P::maker,
        }
    }

    pub fn with_color(mut self, index: usize, color: &'a ColorAttachment) -> Self {
        self.set_color(index, color);
        self
    }

    pub fn set_color(&mut self, index: usize, color: &'a ColorAttachment) {
        self.colors[index] = Some(color);
    }

    pub fn with_depth(mut self, depth_stencil: &'a DepthStencilAttachment) -> Self {
        self.set_depth(depth_stencil);
        self
    }

    pub fn set_depth(&mut self, depth_stencil: &'a DepthStencilAttachment) {
        match self.depth_stencil {
            Some((ref mut attachment, _)) => *attachment = Some(depth_stencil),
            None => {}
        }
    }

    /// Build `PassNode`
    pub(crate) fn build(
        &self,
        device: &B::Device,
        inputs: &[InputAttachmentDesc<B>],
        colors: &[ColorAttachmentDesc<B>],
        depth_stencil: Option<DepthStencilAttachmentDesc<B>>,
        extent: Extent,
    ) -> Result<PassNode<B>> {
        let pass = (self.maker)();        
        // Check attachments
        assert_eq!(inputs.len(), pass.inputs());
        assert_eq!(colors.len(), pass.colors());
        assert_eq!(depth_stencil.is_some(), (pass.depth() || pass.stencil()));

        assert!(inputs.iter().map(|input| Some(input.format)).eq(
            self.inputs.iter().map(|input| input.map(|input| input.format()))
        ));
        assert!(colors.iter().map(|color| Some(color.format)).eq(
            self.colors.iter().map(|color| color.map(|color| color.format))
        ));
        assert_eq!(depth_stencil.as_ref().map(|depth| Some(depth.format)),
            self.depth_stencil.as_ref().map(|depth| depth.0.map(|depth| depth.format))
        );

        // Construct `RenderPass`
        // with single `Subpass` for now
        let render_pass = {
            // Configure input attachments first
            let inputs = inputs.iter().map(|input| {
                pass::Attachment {
                    format: Some(input.format),
                    ops: pass::AttachmentOps {
                        load: pass::AttachmentLoadOp::Load,
                        store: pass::AttachmentStoreOp::Store,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: image::ImageLayout::General..image::ImageLayout::General,
                }
            });

            // Configure color attachments next to input
            let colors = colors.iter().map(|color| {
                pass::Attachment {
                    format: Some(color.format),
                    ops: pass::AttachmentOps {
                        load: if color.clear.is_some() {
                            pass::AttachmentLoadOp::Clear
                        } else {
                            pass::AttachmentLoadOp::Load
                        },
                        store: pass::AttachmentStoreOp::Store,
                    },
                    stencil_ops: pass::AttachmentOps::DONT_CARE,
                    layouts: if color.clear.is_some() {
                        image::ImageLayout::Undefined
                    } else {
                        image::ImageLayout::General
                    }..image::ImageLayout::General,
                }
            });

            // Configure depth-stencil attachments last
            let depth_stencil = depth_stencil.as_ref().map(|depth_stencil| {
                let attachment = pass::Attachment {
                    format: Some(depth_stencil.format),
                    ops: pass::AttachmentOps {
                        load: if pass.depth() && depth_stencil.clear.is_some() {
                            pass::AttachmentLoadOp::Clear
                        } else if pass.depth() {
                            pass::AttachmentLoadOp::Load
                        } else {
                            pass::AttachmentLoadOp::DontCare
                        },
                        store: if pass.depth() {
                            pass::AttachmentStoreOp::Store
                        } else {
                            pass::AttachmentStoreOp::DontCare
                        }
                    },
                    stencil_ops: pass::AttachmentOps {
                        load: if pass.stencil() {
                            pass::AttachmentLoadOp::Load
                        } else {
                            pass::AttachmentLoadOp::DontCare
                        },
                        store: if pass.stencil() {
                            pass::AttachmentStoreOp::Store
                        } else {
                            pass::AttachmentStoreOp::DontCare
                        }
                    },
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

        let descriptors = DescriptorPool::new(&pass.bindings(), device);

        // Create `PipelineLayout` from single `DescriptorSetLayout`
        let pipeline_layout = device.create_pipeline_layout(&[descriptors.layout()], &[]);

        let mut shaders = SmallVec::new();
        // Create `GraphicsPipeline`
        let graphics_pipeline = {
            // Init basic configuration
            let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                pass.shaders(&mut shaders, device)?,
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
                    pass.colors()
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

        for module in shaders {
            device.destroy_shader_module(module);
        }

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
                        AttachmentImageViews::Owned(ref images) => {
                            images
                        }
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
        })
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

/// Get dependencies of pass.
pub fn direct_dependencies<'a, B>(passes: &'a [&'a PassBuilder<'a, B>], pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut deps = Vec::new();
    for input in &pass.inputs {
        let input = input.unwrap();
        deps.extend(passes.iter().filter(|p| {
            p.depth_stencil.as_ref().map(|&(a, _)| input.is_depth(a.unwrap())).unwrap_or(false) ||
            p.colors.iter().any(|a| input.is_color(a.unwrap()))
        }));
    }
    deps.sort_by_key(|p| p as *const _);
    deps.dedup_by_key(|p| p as *const _);
    deps
}

/// Get dependencies of pass. And dependencies of dependencies.
pub fn dependencies<'a, B>(passes: &'a [&'a PassBuilder<'a, B>], pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut deps = direct_dependencies(passes, pass);
    deps = deps.into_iter().flat_map(|deps| dependencies(passes, deps)).collect();
    deps.sort_by_key(|p| p as *const _);
    deps.dedup_by_key(|p| p as *const _);
    deps
}

/// Get dependencies of pass that aren't dependency of dependency.
pub fn linear_dependencies<'a, B>(passes: &'a [&'a PassBuilder<'a, B>], pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut alldeps = direct_dependencies(passes, pass);
    let mut newdeps = vec![];
    while let Some(dep) = alldeps.pop() {
        newdeps.push(dep);
        let other = dependencies(passes, dep);
        alldeps.retain(|dep| indices_in_of(&other, &[dep]).is_none());
        newdeps.retain(|dep| indices_in_of(&other, &[dep]).is_none());
    }
    newdeps
}

/// Get all color attachments for all passes
pub fn color_attachments<'a, B>(passes: &[&PassBuilder<'a, B>]) -> Vec<&'a ColorAttachment>
where
    B: Backend,
{
    let mut attachments = Vec::new();
    for pass in passes {
        attachments.extend(pass.colors.iter().cloned().map(Option::unwrap));
    }
    attachments.sort_by_key(|a| a.key());
    attachments.dedup_by_key(|a| a.key());
    attachments
}

/// Get all depth_stencil attachments for all passes
pub fn depth_stencil_attachments<'a, B>(passes: &[&PassBuilder<'a, B>]) -> Vec<&'a DepthStencilAttachment>
where
    B: Backend,
{
    let mut attachments = Vec::new();
    for pass in passes {
        attachments.extend(pass.depth_stencil.as_ref().map(|&(a, _)| a.unwrap()));
    }
    attachments.sort_by_key(|a| a.key());
    attachments.dedup_by_key(|a| a.key());
    attachments
}


/// Get other passes that shares output attachments
pub fn siblings<'a, B>(passes: &'a [&'a PassBuilder<'a, B>], pass: &'a PassBuilder<'a, B>) -> Vec<&'a PassBuilder<'a, B>>
where
    B: Backend,
{
    let mut siblings = Vec::new();
    for &color in pass.colors.iter() {
        siblings.extend(passes.iter().filter(|p| {
            p.colors.iter().any(|a| (a.unwrap().key()) == (color.unwrap().key()))
        }));
    }
    if let Some((Some(depth), _)) = pass.depth_stencil {
        siblings.extend(passes.iter().filter(|p| {
            p.depth_stencil.as_ref().map(|&(a, _)| (a.unwrap().key()) ==  (depth.key())).unwrap_or(false)
        }));
    }
    siblings.sort_by_key(|p| p as *const _);
    siblings.dedup_by_key(|p| p as *const _);
    siblings
}

pub fn create_target<B>(
    format: Format,
    allocator: &mut Allocator<B>,
    device: &B::Device,
    images: &mut Vec<Image<B>>,
    views: &mut Vec<B::ImageView>,
    extent: Extent,
    frames: usize,
) -> Result<Range<usize>>
where
    B: Backend,
{
    let kind = image::Kind::D2(
        extent.width as u16,
        extent.height as u16,
        image::AaMode::Single,
    );
    let start = views.len();
    for _ in 0..frames {
        let image = allocator.create_image(
            device,
            kind,
            1,
            format,
            image::Usage::COLOR_ATTACHMENT,
            Properties::DEVICE_LOCAL,
        )?;
        let view = device.create_image_view(image.raw(), format, Swizzle::NO, COLOR_RANGE.clone())?;
        views.push(view);
        images.push(image);
    }
    Ok(start..views.len())
}

pub fn reorder_passes<'a, B>(
    passes: &[&'a PassBuilder<'a, B>],
) -> (Vec<&'a PassBuilder<'a, B>>, Vec<Option<usize>>)
where
    B: Backend,
{
    let mut unscheduled: Vec<_> = passes.iter().cloned().collect();
    // Ordered passes
    let mut scheduled = vec![];
    let mut deps = vec![];

    // Until we schedule all unscheduled passes
    while !unscheduled.is_empty() {
        // Walk over unscheduled
        let (last_dep, index) = (0..unscheduled.len())
            .filter_map(|index| {
                // Sanity check. This pass wasn't scheduled yet
                debug_assert_eq!(None, indices_in_of(&scheduled, &[unscheduled[index]]));
                // Find indices for all direct dependencies of the pass
                indices_in_of(&scheduled, &direct_dependencies(passes, unscheduled[index]))
                    .map(|deps| {
                        // Add all already scheduled passes that shares some outputs
                        let siblings = siblings(&scheduled, unscheduled[index]);
                        let siblings = some_indices_in_of(&scheduled, &siblings);
                        (deps.into_iter().chain(siblings).max(), index)
                    })
            })
            // Smallest index of last dependency wins. `None < Some(0)`
            .min_by_key(|&(last_dep, _)| last_dep)
            // At least one pass with all dependencies scheduled must be found.
            // Or there is dependency circle in unscheduled left.
            .expect("Circular dependency encountered");

        // Sanity check. All dependencies must be scheduled if all direct dependencies are
        debug_assert!(indices_in_of(&scheduled, &dependencies(passes, unscheduled[index])).is_some());

        // Store
        scheduled.push(unscheduled[index]);
        deps.push(last_dep);

        // remove from unscheduled
        unscheduled.swap_remove(index);
    }
    (scheduled, deps)
}
