pub mod build;
pub mod pass;
pub mod flat;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::iter::Empty;
use std::marker::PhantomData;
use std::mem::{replace, transmute};
use std::ops::Range;

use gfx_hal::Backend;
use gfx_hal::command::{ClearValue, CommandBuffer, Rect, Viewport};
use gfx_hal::device::{Device, Extent, FramebufferError, WaitFor};
use gfx_hal::format::{Format, Swizzle};
use gfx_hal::image;
use gfx_hal::memory::Properties;
use gfx_hal::pool::CommandPool;
use gfx_hal::pso::{BlendState, CreationError, PipelineStage};
use gfx_hal::queue::CommandQueue;
use gfx_hal::queue::capability::{Graphics, Supports, Transfer};
use gfx_hal::window::{Backbuffer, Frame, Swapchain};

use smallvec::SmallVec;
use specs::World;

use epoch::{CurrentEpoch, Epoch};
use graph::pass::AnyPass;
use memory::{Allocator, Image};
use shaders::ShaderManager;

pub use graph::build::*;


error_chain!{
    errors {
        FramebufferError {
            description("Failed to create framebuffer")
            display("Failed to create framebuffer")
        }
    }

    links {
        Memory(::memory::Error, ::memory::ErrorKind);
        Shader(::shaders::Error, ::shaders::ErrorKind);
    }

    foreign_links {
        CreationError(CreationError);
        ViewError(image::ViewError);
    }
}


const COLOR_RANGE: image::SubresourceRange = image::SubresourceRange {
    aspects: image::AspectFlags::COLOR,
    levels: 0..1,
    layers: 0..1,
};


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
    fn new(backbuffer: &'a Backbuffer<B>, frame: Frame) -> Self {
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
    descriptor_set_layout: B::DescriptorSetLayout,
    pipeline_layout: B::PipelineLayout,
    graphics_pipeline: B::GraphicsPipeline,
    render_pass: B::RenderPass,
    framebuffer: SuperFramebuffer<B>,
    pass: Box<AnyPass<B>>,
    depends: Vec<(usize, PipelineStage)>,
    draws_to_surface: bool,
}

impl<B> PassNode<B>
where
    B: Backend,
{
    /// Record drawing command for this node.
    fn draw<C>(
        &mut self,
        cbuf: &mut CommandBuffer<B, C>,
        device: &B::Device,
        world: &World,
        rect: Rect,
        frame: SuperFrame<B>,
        through: Epoch,
    ) -> bool
    where
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
        {
            profile_scope!("AnyPass::prepare");
            self.pass.prepare(
                through,
                cbuf.downgrade(),
                &self.pipeline_layout,
                device,
                world,
            );
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
        self.pass.draw_inline(through, encoder, world);

        // Return if this pass renders to the surface image
        self.draws_to_surface
    }
}

/// Directed acyclic rendering graph.
/// It holds all rendering nodes and auxilary data.
#[derive(Debug)]
pub struct Graph<B: Backend> {
    passes: Vec<PassNode<B>>,
    signals: Vec<B::Semaphore>,
    acquire: B::Semaphore,
    images: Vec<Image<B>>,
    views: Vec<B::ImageView>,
    frames: usize,
}

impl<B> Graph<B>
where
    B: Backend,
{
    /// Get number of frames that can be renderered in parallel with this graph
    pub fn get_frames_number(&self) -> usize {
        self.frames
    }

    /// Walk over graph recording drawing commands and submitting them to `queue`.
    /// This function handles synchronization between dependent rendering nodes.
    ///
    /// `queue` must come from same `QueueGroup` with witch `pool` is associated.
    /// `swapchain` must be created with `backbuffer`.
    /// `backbuffer` must be the same that was used in `build` function.
    /// All those should be created by `device`.
    pub fn draw<S, C>(
        &mut self,
        queue: &mut CommandQueue<B, C>,
        pool: &mut CommandPool<B, C>,
        swapchain: &mut S,
        backbuffer: &Backbuffer<B>,
        device: &B::Device,
        viewport: Viewport,
        world: &World,
        finish: &B::Fence,
        through: Epoch,
    ) where
        S: Swapchain<B>,
        C: Supports<Graphics> + Supports<Transfer>,
    {
        use gfx_hal::image::ImageLayout;
        use gfx_hal::queue::submission::Submission;
        use gfx_hal::window::FrameSync;
        use std::iter::once;

        profile_scope!("Graph::draw");

        let ref signals = self.signals;
        let ref acquire = self.acquire;
        let count = self.passes.len();

        // Start frame acquisition
        let frame = SuperFrame::new(
            backbuffer,
            swapchain.acquire_frame(FrameSync::Semaphore(acquire)),
            // Frame::new(0),
        );

        // Store `Semaphore`s that `Surface::present` needs to wait for
        let mut presents: SmallVec<[_; 32]> = SmallVec::new();

        // Record commands for all passes
        self.passes.iter_mut().enumerate().for_each(|(id, pass)| {
            profile_scope!("Graph::draw :: pass");
            // Pick buffer
            let mut cbuf = pool.acquire_command_buffer();

            // Setup
            cbuf.set_viewports(&[viewport.clone()]);
            cbuf.set_scissors(&[viewport.rect]);

            // Record commands for pass
            let draws_to_surface =
                pass.draw(&mut cbuf, device, world, viewport.rect, frame.clone(), through);

            // If it renders to acquired image
            let wait_surface = if draws_to_surface {
                // Presenting has to wait for it
                presents.push(&signals[id]);
                // And it should wait for acqusition
                Some((acquire, PipelineStage::TOP_OF_PIPE))
            } else {
                None
            };

            {
                profile_scope!("Graph::draw :: pass :: submit");
                // Submit buffer
                queue.submit(
                    Submission::new()
                        .promote::<C>()
                        .submit(&[cbuf.finish()])
                        .wait_on(&pass.depends
                            .iter()
                            .map(|&(id, stage)| (&signals[id], stage))
                            .chain(wait_surface)
                            .collect::<SmallVec<[_; 32]>>()) // Each pass singnals to associated `Semaphore`
                        .signal(&[&signals[id]]),
                    // Signal the finish fence in last submission
                    if id == count - 1 { Some(finish) } else { None },
                );
            }
        });

        // Present queue
        profile_scope!("Graph::draw :: present");
        swapchain.present(queue, &presents);
    }


    /// Build rendering graph from `ColorPin`
    /// for specified `backbuffer`.
    pub fn build(
        present: ColorPin<B>,
        backbuffer: &Backbuffer<B>,
        color: Format,
        depth_stencil: Option<Format>,
        extent: Extent,
        device: &B::Device,
        allocator: &mut Allocator<B>,
        shaders: &mut ShaderManager<B>,
    ) -> Result<Self> {
        assert_eq!(present.format(), color);

        // Create views for backbuffer
        let (mut image_views, frames) = match *backbuffer {
            Backbuffer::Images(ref images) => (
                images
                    .iter()
                    .map(|image| {
                        device
                            .create_image_view(image, color, Swizzle::NO, COLOR_RANGE.clone())
                            .map_err(Into::into)
                    })
                    .collect::<Result<Vec<_>>>()?,
                images.len(),
            ),
            Backbuffer::Framebuffer(_) => (vec![], 1),
        };

        // Collect all passes by walking dependecy tree
        let passes = traverse(&present);

        // Reorder passes to increase overlapping
        let passes = {
            let mut unscheduled = passes;
            // Ordered passes
            let mut scheduled = vec![];

            // Same scheduled passes but with dependencies as indices
            let mut passes = vec![];

            // Until we schedule all unscheduled passes
            while !unscheduled.is_empty() {
                // Walk over unscheduled
                let (deps, index) = (0..unscheduled.len())
                    .filter_map(|index| {
                        // Sanity check. This pass wasn't scheduled yet
                        assert_eq!(None, dependency_search(&scheduled, &[unscheduled[index]]));
                        // Find indices for all direct dependencies of the pass
                        dependency_search(&scheduled, &direct_dependencies(unscheduled[index]))
                            .map(|deps| (deps, index))
                    })
                    // Smallest index of last dependency wins. `None < Some(0)`
                    .min_by_key(|&(ref deps, _)| deps.last().cloned())
                    // At least one pass with all dependencies scheduled must be found.
                    // Or there is dependency circle in unscheduled left.
                    .expect("Circular dependency encountered");

                // Sanity check. All dependencies must be scheduled if all direct dependencies are
                assert!(dependency_search(&scheduled, &dependencies(unscheduled[index])).is_some());

                // Store
                scheduled.push(unscheduled[index]);
                passes.push((unscheduled[index], deps));

                // remove from unscheduled
                unscheduled.swap_remove(index);
            }
            passes
        };

        // Get all merges
        let merges = merges(&present);

        // Setup image storage
        let mut images = vec![];

        // Initialize all targets
        let mut targets = HashMap::<*const _, Targets>::new();
        for &merge in merges.iter() {
            let present_key = present.merge as *const _;
            let key = merge as *const _;
            targets.insert(
                key,
                create_targets(
                    allocator,
                    device,
                    merge,
                    &mut images,
                    &mut image_views,
                    extent,
                    frames,
                    |index| key == present_key && present.index == index,
                )?,
            );
        }

        // Build pass nodes from pass builders
        let passes: Vec<PassNode<B>> = passes
            .into_iter()
            .map(|(pass, deps)| {
                // Collect input targets
                let inputs = pass.connects
                    .iter()
                    .map(|pin| {
                        let (indices, format) = match *pin {
                            Pin::Color(ColorPin { merge, index }) => (
                                targets.get(&(merge as *const _)).unwrap().colors[index]
                                    .indices
                                    .clone()
                                    .unwrap(),
                                merge.color_format(index),
                            ),
                            Pin::Depth(DepthPin { merge }) => (
                                targets
                                    .get(&(merge as *const _))
                                    .and_then(|targets| targets.depth.as_ref())
                                    .map(|depth| depth.indices.clone())
                                    .unwrap(),
                                merge.depth_format().unwrap(),
                            ),
                        };
                        let ref view = image_views[indices];
                        InputAttachmentDesc { format, view }
                    })
                    .collect::<Vec<_>>();

                // Find where this pass going
                let merge = *merges
                    .iter()
                    .find(|&merge| {
                        merge
                            .passes
                            .iter()
                            .any(|&p| p as *const _ == pass as *const _)
                    })
                    .expect("All passes comes from merges");
                let key = merge as *const _;

                let is_first = merge.passes[0] as *const _ == pass as *const _;
                let clear_color = if is_first { merge.clear_color } else { None };
                let clear_depth = if is_first { merge.clear_depth } else { None };

                // Try to guess why this line is absolutely required.
                let ref image_views = image_views;

                // Collect color targets
                let ref target = targets[&key];
                let colors = target
                    .colors
                    .iter()
                    .enumerate()
                    .map(|(attachment_index, color_indices)| {
                        // Get image view and format
                        let (view, format) = color_indices
                            .indices
                            .clone()
                            .map(|indices| {
                                (
                                    // It's owned image
                                    AttachmentImageView::Owned(&image_views[indices]),
                                    merge.color_format(attachment_index),
                                )
                            })
                            .unwrap_or_else(|| {
                                (
                                    // It's backbuffer image
                                    match *backbuffer {
                                        Backbuffer::Images(_) => {
                                            AttachmentImageView::Acquired(&image_views[0..frames])
                                        }
                                        Backbuffer::Framebuffer(_) => AttachmentImageView::External,
                                    },
                                    color,
                                )
                            });
                        ColorAttachmentDesc {
                            format,
                            view,
                            clear: clear_color,
                        }
                    })
                    .collect::<Vec<_>>();

                let depth_stencil = target.depth.clone().map(|depth| {
                    DepthStencilAttachmentDesc {
                        format: merge.depth_format().unwrap(),
                        view: AttachmentImageView::Owned(&image_views[depth.indices]),
                        clear: clear_depth,
                    }
                });

                let mut node = pass.build(
                    device,
                    shaders,
                    &inputs[..],
                    &colors[..],
                    depth_stencil,
                    extent,
                )?;

                node.depends = deps.into_iter()
                    .map(|dep| {
                        (dep, PipelineStage::TOP_OF_PIPE) // Pick better
                    })
                    .collect();
                Ok(node)
            })
            .collect::<Result<_>>()?;

        let count = passes.len();

        Ok(Graph {
            passes: passes,
            signals: (0..count).map(|_| device.create_semaphore()).collect(),
            acquire: device.create_semaphore(),
            images,
            views: image_views,
            frames,
        })
    }
}

#[derive(Clone)]
struct Targets {
    colors: Vec<ColorIndices>,
    depth: Option<DepthIndices>,
}

#[derive(Clone)]
struct ColorIndices {
    indices: Option<Range<usize>>,
}

#[derive(Clone)]
struct DepthIndices {
    indices: Range<usize>,
}


fn create_targets<B, F>(
    allocator: &mut Allocator<B>,
    device: &B::Device,
    merge: &Merge<B>,
    images: &mut Vec<Image<B>>,
    views: &mut Vec<B::ImageView>,
    extent: Extent,
    frames: usize,
    f: F,
) -> Result<Targets>
where
    B: Backend,
    F: Fn(usize) -> bool,
{
    let mut make_views = |format: Format| -> Result<Range<usize>> {
        let kind = image::Kind::D2(
            extent.width as u16,
            extent.height as u16,
            image::AaMode::Single,
        );
        let start = views.len();
        for i in 0..frames {
            let image = allocator.create_image(
                device,
                kind,
                1,
                format,
                image::Usage::COLOR_ATTACHMENT,
                Properties::DEVICE_LOCAL,
            )?;
            let view =
                device.create_image_view(image.raw(), format, Swizzle::NO, COLOR_RANGE.clone())?;
            views.push(view);
            images.push(image);
        }
        Ok(start..views.len())
    };

    let colors = (0..merge.colors())
        .map(|i| -> Result<_> {
            Ok(if f(i) {
                ColorIndices { indices: None }
            } else {
                ColorIndices {
                    indices: Some(make_views(merge.color_format(i))?),
                }
            })
        })
        .collect::<Result<_>>()?;

    Ok(if let Some(format) = merge.depth_format() {
        Targets {
            colors,
            depth: Some(DepthIndices {
                indices: make_views(format)?,
            }),
        }
    } else {
        Targets {
            colors,
            depth: None,
        }
    })
}
