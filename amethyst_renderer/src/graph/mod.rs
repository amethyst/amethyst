//!
//! Defines `Graph` - complex rendering graph.
//! And `Pass` - building block for `Graph`.
//! TODO: compute.

mod build;
mod pass;

use std::collections::HashMap;
use std::ops::Range;

use failure::ResultExt;

use gfx_hal::Backend;
use gfx_hal::command::{ClearValue, CommandBuffer, Rect, Viewport};
use gfx_hal::device::{Device, Extent, FramebufferError, ShaderError};
use gfx_hal::format::{Format, Swizzle};
use gfx_hal::image;
use gfx_hal::memory::Properties;
use gfx_hal::pool::CommandPool;
use gfx_hal::pso::{CreationError, PipelineStage};
use gfx_hal::queue::CommandQueue;
use gfx_hal::queue::capability::{Graphics, Supports, Transfer};
use gfx_hal::window::{Backbuffer, Frame};

use shred::Resources;
use smallvec::SmallVec;
use specs::World;

use descriptors::DescriptorPool;
use epoch::Epoch;
use graph::build::*;
use graph::pass::AnyPass;
use memory::{Allocator, Image};

pub use graph::build::{ColorAttachment, DepthStencilAttachment, PassBuilder};
pub use graph::pass::{Data, Pass, PassTag};

#[derive(Fail, Debug, Clone)]
pub enum Error {
    #[fail(display = "Failed to create framebuffer")] FramebufferError,

    #[fail(display = "Shader compilation failed: {}", msg)] CompilationFailed { msg: String },

    #[fail(display = "Missing shader entry point: {}", msg)] MissingEntryPoint { msg: String },

    #[fail(display = "Shader interface mismatch: {}", msg)] InterfaceMismatch { msg: String },
}

impl From<FramebufferError> for Error {
    fn from(_: FramebufferError) -> Error {
        Error::FramebufferError
    }
}

impl From<ShaderError> for Error {
    fn from(error: ShaderError) -> Error {
        match error {
            ShaderError::CompilationFailed(msg) => Error::CompilationFailed { msg }.into(),
            ShaderError::MissingEntryPoint(msg) => Error::MissingEntryPoint { msg }.into(),
            ShaderError::InterfaceMismatch(msg) => Error::InterfaceMismatch { msg }.into(),
        }
    }
}

/// This wrapper allow to abstract over two cases.
/// `Index` => index to one of multiple `Framebuffer`s created by the engine.
/// `Buffer` => Single `Framebuffer` associated with `Swapchain`.
#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub enum SuperFrame<'a, B: Backend> {
    Index(usize),
    Buffer(&'a B::Framebuffer),
}

impl<'a, B> SuperFrame<'a, B>
where
    B: Backend,
{
    /// Create `SuperFrame` from `Backbuffer` and `Frame` index.
    pub fn new(backbuffer: &'a Backbuffer<B>, frame: Frame) -> Self {
        // Check if we have `Framebuffer` from `Surface` (usually with OpenGL backend) or `Image`s
        // In case it's `Image`s we need to pick `Framebuffer` for `RenderPass`es
        // that renders onto surface.
        match *backbuffer {
            Backbuffer::Images(_) => SuperFrame::Index(frame.id()),
            Backbuffer::Framebuffer(ref single) => SuperFrame::Buffer(single),
        }
    }
}

/// Framebuffer wrapper
#[derive(Debug)]
pub enum SuperFramebuffer<B: Backend> {
    /// Target is multiple `Framebuffer`s created over `ImageView`s
    Owned(Vec<B::Framebuffer>),

    /// Target is single `Framebuffer` associated with `Swapchain`
    External,
}

/// Picks correct framebuffer
fn pick<'a, B>(framebuffer: &'a SuperFramebuffer<B>, frame: SuperFrame<'a, B>) -> &'a B::Framebuffer
where
    B: Backend,
{
    use self::SuperFrame::*;
    use self::SuperFramebuffer::*;
    match (framebuffer, frame) {
        (&Owned(ref framebuffers), Index(index)) => &framebuffers[index],
        (&Owned(ref framebuffers), Buffer(_)) => &framebuffers[0],
        (&External, Buffer(ref framebuffer)) => framebuffer,
        _ => unreachable!("This combination can't happen"),
    }
}

/// Single node in rendering graph.
/// Nodes can use output of other nodes as input.
/// Such connection called `dependency`
#[derive(Debug)]
pub struct PassNode<B: Backend> {
    clears: Vec<ClearValue>,
    descriptors: DescriptorPool<B>,
    pipeline_layout: B::PipelineLayout,
    graphics_pipeline: B::GraphicsPipeline,
    render_pass: B::RenderPass,
    framebuffer: SuperFramebuffer<B>,
    pass: Box<AnyPass<B>>,
    depends: Option<(usize, PipelineStage)>,
}

impl<B> PassNode<B>
where
    B: Backend,
{
    /// Binds pipeline and renderpass to the command buffer `cbuf`.
    /// Executes `Pass::prepare` and `Pass::draw_inline` of the inner `Pass`
    /// to record transfer and draw commands.
    ///
    /// `res` - primary source of data (`Mesh`es, `Texture`s etc) for the `Pass`es.
    /// `rect` - area to draw in.
    /// `frame` - specifies which framebuffer and descriptor sets to use.
    /// `span` - commands will be executed in specified epoch range. Pass should ensure all resources
    /// are valid for execution span.
    ///
    fn draw_inline<C>(
        &mut self,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        res: &Resources,
        rect: Rect,
        frame: SuperFrame<B>,
        span: Range<Epoch>,
    ) where
        C: Supports<Graphics> + Supports<Transfer>,
    {
        profile_scope!("PassNode::draw");
        // Bind pipeline

        {
            profile_scope!("PassNode::draw :: bind_graphics_pipeline");
            cbuf.bind_graphics_pipeline(&self.graphics_pipeline);
        }

        // Run custom preparation
        // * Write descriptor sets
        // * Store caches
        // * Bind pipeline layout with descriptors sets
        {
            profile_scope!("AnyPass::prepare");
            self.pass
                .prepare(span.clone(), allocator, device, cbuf.downgrade(), res);
        }

        let encoder = {
            profile_scope!("PassNode::draw :: begin_renderpass_inline");
            // Begin render pass with single inline subpass
            cbuf.begin_renderpass_inline(
                &self.render_pass,
                pick(&self.framebuffer, frame),
                rect,
                &self.clears,
            )
        };

        profile_scope!("AnyPass::draw_inline");
        // Record custom drawing calls
        self.pass.draw_inline(
            span,
            &self.pipeline_layout,
            &mut self.descriptors,
            device,
            encoder,
            res,
        );
    }

    fn dispose(mut self, allocator: &mut Allocator<B>, device: &B::Device, res: &Resources) {
        self.pass.cleanup(&mut self.descriptors, res);
        match self.framebuffer {
            SuperFramebuffer::Owned(framebuffers) => for framebuffer in framebuffers {
                device.destroy_framebuffer(framebuffer);
            },
            _ => {}
        }
        device.destroy_renderpass(self.render_pass);
        device.destroy_graphics_pipeline(self.graphics_pipeline);
        device.destroy_pipeline_layout(self.pipeline_layout);
    }
}

/// Directed acyclic rendering graph.
/// It holds all rendering nodes and auxiliary data.
#[derive(Debug)]
pub struct Graph<B: Backend> {
    passes: Vec<PassNode<B>>,
    signals: Vec<Option<B::Semaphore>>,
    images: Vec<Image<B>>,
    views: Vec<B::ImageView>,
    frames: usize,
    first_draws_to_surface: usize,
}

impl<B> Graph<B>
where
    B: Backend,
{
    /// Get number of frames that can be rendered in parallel with this graph
    pub fn get_frames_number(&self) -> usize {
        self.frames
    }

    /// Walk over graph recording drawing commands and submitting them to `queue`.
    /// This function handles synchronization between dependent rendering nodes.
    ///
    /// `queue` must come from same `QueueGroup` with which `pool` is associated.
    /// All those should be created by `device`.
    ///
    /// `frame` - frame index that should be drawn.
    /// `upload` - semaphore that will be signaled when all data will be uploaded.
    /// (or `Framebuffer` reference that corresponds to index `0`)
    /// `acquire` - semaphore that should be waited on by submissions which
    /// contains commands from passes that draw to the surface
    /// `device` - you need this guy everywhere =^_^=
    /// `viewport` - portion of framebuffers to draw
    /// `res` - primary source of stuff to draw
    /// `finish` - last submission should set this fence
    /// `span` - all commands will be finished before this epoch ends.
    pub fn draw_inline<C>(
        &mut self,
        queue: &mut CommandQueue<B, C>,
        pool: &mut CommandPool<B, C>,
        frame: SuperFrame<B>,
        upload: Option<&B::Semaphore>,
        acquire: &B::Semaphore,
        release: &B::Semaphore,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        viewport: Viewport,
        res: &Resources,
        finish: &B::Fence,
        span: Range<Epoch>,
    ) where
        C: Supports<Graphics> + Supports<Transfer>,
    {
        use gfx_hal::queue::submission::Submission;

        profile_scope!("Graph::draw");
        let ref signals = self.signals;
        let count = self.passes.len();
        let first_draws_to_surface = self.first_draws_to_surface;

        // Record commands for all passes
        self.passes.iter_mut().enumerate().for_each(|(id, pass)| {
            profile_scope!("Graph::draw :: pass");
            // Pick buffer
            let mut cbuf = pool.acquire_command_buffer();

            // Setup
            cbuf.set_viewports(&[viewport.clone()]);
            cbuf.set_scissors(&[viewport.rect]);

            // Record commands for pass
            pass.draw_inline(
                &mut cbuf,
                allocator,
                device,
                res,
                viewport.rect,
                frame.clone(),
                span.clone(),
            );

            {
                profile_scope!("Graph::draw :: pass :: submit");
                // If it renders to acquired image
                let wait_surface = if id == first_draws_to_surface {
                    // And it should wait for acquisition
                    Some((acquire, PipelineStage::TOP_OF_PIPE))
                } else {
                    None
                };

                let wait_for_upload = if id == 0 {
                    upload.map(|upload| (upload, PipelineStage::TOP_OF_PIPE))
                } else {
                    None
                };

                let to_wait = pass.depends
                    .as_ref()
                    .map(|&(id, stage)| (signals[id].as_ref().unwrap(), stage))
                    .into_iter()
                    .chain(wait_surface)
                    .chain(wait_for_upload)
                    .collect::<SmallVec<[_; 3]>>();

                let mut to_signal = SmallVec::<[_; 1]>::new();
                if id == count - 1 {
                    // The last one has to draw to surface.
                    // Also it depends on all others that draws to surface.
                    to_signal.push(release);
                } else if let Some(signal) = signals[id].as_ref() {
                    to_signal.push(signal);
                };

                // Signal the finish fence in last submission
                let fence = if id == count - 1 { Some(finish) } else { None };

                // Submit buffer
                queue.submit(
                    Submission::new()
                        .promote::<C>()
                        .submit(&[cbuf.finish()])
                        .wait_on(&to_wait)
                        .signal(&to_signal),
                    fence,
                );
            }
        });
    }

    /// Build rendering graph from `ColorPin`
    /// for specified `backbuffer`.
    pub fn build(
        passes: &[&PassBuilder<B>],
        present: &ColorAttachment,
        backbuffer: &Backbuffer<B>,
        extent: Extent,
        device: &B::Device,
        allocator: &mut Allocator<B>,
    ) -> Result<Self, ::failure::Error> {
        // Create views for backbuffer
        let (mut image_views, frames) = match *backbuffer {
            Backbuffer::Images(ref images) => (
                images
                    .iter()
                    .map(|image| {
                        device.create_image_view(
                            image,
                            present.format,
                            Swizzle::NO,
                            COLOR_RANGE.clone(),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .with_context(|err| format!("Failed to build graph: {}", err))?,
                images.len(),
            ),
            Backbuffer::Framebuffer(_) => (vec![], 1),
        };

        // Reorder passes to maximise overlapping
        // while keeping all dependencies before dependants.
        let (passes, deps) = reorder_passes(passes);

        let color_attachments = color_attachments(&passes);
        let depth_stencil_attachments = depth_stencil_attachments(&passes);

        // Setup image storage
        let mut images = vec![];

        // Initialize all targets
        let mut color_targets = HashMap::<*const (), (Range<usize>, usize)>::new();
        let present_key = present.key();
        color_targets.insert(present_key, (0..image_views.len(), 0));
        for &attachment in color_attachments.iter() {
            let key = attachment.key();
            if key != present_key {
                color_targets.insert(
                    key,
                    (
                        create_target(
                            attachment.format,
                            allocator,
                            device,
                            &mut images,
                            &mut image_views,
                            extent,
                            frames,
                        )?,
                        0,
                    ),
                );
            }
        }

        let mut depth_stencil_targets = HashMap::<*const (), (Range<usize>, usize)>::new();
        for &attachment in depth_stencil_attachments.iter() {
            let key = attachment.key();
            depth_stencil_targets.insert(
                key,
                (
                    create_target(
                        attachment.format,
                        allocator,
                        device,
                        &mut images,
                        &mut image_views,
                        extent,
                        frames,
                    )?,
                    0,
                ),
            );
        }

        // Build pass nodes from pass builders
        let mut pass_nodes: Vec<PassNode<B>> = Vec::new();

        let mut first_draws_to_surface = None;

        for (id, (pass, last_dep)) in passes.into_iter().zip(deps).enumerate() {
            // Collect input targets
            let inputs = pass.inputs
                .iter()
                .map(|input| {
                    let input = input.unwrap();
                    let (ref indices, ref written) = *match input {
                        Attachment::Color(color) => &color_targets[&color.key()],
                        Attachment::DepthStencil(depth_stencil) => {
                            &depth_stencil_targets[&depth_stencil.key()]
                        }
                    };
                    let indices = indices.clone();
                    debug_assert!(*written > 0);
                    let ref view = image_views[indices];
                    InputAttachmentDesc {
                        format: input.format(),
                        view,
                    }
                })
                .collect::<Vec<_>>();

            let colors = pass.colors
                .iter()
                .enumerate()
                .map(|(index, color)| {
                    let color = color.unwrap();
                    if first_draws_to_surface.is_none() && (color.key()) == present_key {
                        first_draws_to_surface = Some(index);
                    }
                    let (ref indices, ref mut written) =
                        *color_targets.get_mut(&color.key()).unwrap();
                    let indices = indices.clone();
                    let clear = if *written == 0 { color.clear } else { None };

                    *written += 1;

                    ColorAttachmentDesc {
                        format: color.format,
                        view: if indices != (0..0) {
                            AttachmentImageViews::Owned(&image_views[indices])
                        } else {
                            AttachmentImageViews::External
                        },
                        clear,
                    }
                })
                .collect::<Vec<_>>();

            let depth_stencil = pass.depth_stencil.clone().map(|(depth, stencil)| {
                let depth = depth.unwrap();
                let (ref indices, ref mut written) =
                    *depth_stencil_targets.get_mut(&depth.key()).unwrap();
                let indices = indices.clone();
                let clear = if *written == 0 { depth.clear } else { None };

                *written += 1;

                DepthStencilAttachmentDesc {
                    format: depth.format,
                    view: if indices != (0..0) {
                        AttachmentImageViews::Owned(&image_views[indices])
                    } else {
                        AttachmentImageViews::External
                    },
                    clear,
                }
            });

            let mut node = pass.build(device, &inputs[..], &colors[..], depth_stencil, extent)?;

            if let Some(last_dep) = last_dep {
                node.depends = if pass_nodes
                    .iter()
                    .find(|node| {
                        node.depends
                            .as_ref()
                            .map(|&(id, _)| id == last_dep)
                            .unwrap_or(false)
                    })
                    .is_none()
                {
                    // No passes prior this depends on `last_dep`
                    Some((last_dep, PipelineStage::TOP_OF_PIPE)) // Pick better
                } else {
                    None
                };
            }

            pass_nodes.push(node);
        }

        let mut signals = Vec::new();
        for i in 0..pass_nodes.len() {
            if let Some(j) = pass_nodes.iter().position(|node| {
                node.depends
                    .as_ref()
                    .map(|&(id, _)| id == i)
                    .unwrap_or(false)
            }) {
                // j depends on i
                assert!(
                    pass_nodes
                        .iter()
                        .skip(j + 1)
                        .find(|node| node.depends
                            .as_ref()
                            .map(|&(id, _)| id == i)
                            .unwrap_or(false))
                        .is_none()
                );
                signals.push(Some(device.create_semaphore()));
            } else {
                signals.push(None);
            }
        }

        Ok(Graph {
            passes: pass_nodes,
            signals,
            images,
            views: image_views,
            frames,
            first_draws_to_surface: first_draws_to_surface.unwrap(),
        })
    }

    pub fn dispose(self, allocator: &mut Allocator<B>, device: &B::Device, res: &Resources) {
        for pass in self.passes {
            pass.dispose(allocator, device, res);
        }
        for signal in self.signals.into_iter().filter_map(|x| x) {
            device.destroy_semaphore(signal);
        }
        for view in self.views {
            device.destroy_image_view(view);
        }
        for image in self.images {
            allocator.destroy_image(image);
        }
    }
}
