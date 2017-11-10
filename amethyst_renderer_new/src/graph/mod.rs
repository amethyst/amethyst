
pub mod build;
pub mod pass;

use std::marker::PhantomData;
use std::mem::replace;
use std::iter::Empty;

use gfx_hal::Backend;
use gfx_hal::command::{ClearValue, Rect, SubpassContents, Viewport};
use gfx_hal::device::{Device, WaitFor};
use gfx_hal::pso::{BlendState, PipelineStage};
use gfx_hal::queue::CommandQueue;
use gfx_hal::queue::capability::{Graphics, Supports, Transfer};
use gfx_hal::window::{Backbuffer, Frame, Swapchain};

use smallvec::SmallVec;

use specs::World;

use self::pass::AnyPass;


error_chain!{
    foreign_links {
        CreationError(::gfx_hal::pso::CreationError);
    }
}


#[derive(Derivative)]
#[derivative(Clone, Debug)]
enum SuperFrame<'a, B: Backend> {
    Index(usize),
    Buffer(&'a B::Framebuffer),
}

impl<'a, B> SuperFrame<'a, B>
where
    B: Backend,
{
    fn new(backbuffer: &'a Backbuffer<B>, frame: Frame) -> Self {
        // Check if we have `Framebuffer` from `Surface` (OpenGL) or `Image`s
        // In case it's `Image`s we need to pick `Framebuffer` for `RenderPass`es
        // that renders onto surface.
        match *backbuffer {
            Backbuffer::Images(_) => SuperFrame::Index(frame.id()),
            Backbuffer::Framebuffer(ref single) => SuperFrame::Buffer(single),
        }
    }
}

#[derive(Debug)]
enum SuperFramebuffer<B: Backend> {
    /// Target is owned by `RenderGraph`
    Owned(B::Framebuffer),

    /// Target is acquired from `Swapchain`
    Acquired(Vec<B::Framebuffer>),

    /// Target is single `Framebuffer`
    Single,
}

impl<B> SuperFramebuffer<B>
where
    B: Backend,
{
    fn is_acquired(&self) -> bool {
        use self::SuperFramebuffer::*;
        match *self {
            Acquired(_) | Single => true,
            Owned(_) => false,
        }
    }
}

fn pick<'a, B>(framebuffer: &'a SuperFramebuffer<B>, frame: SuperFrame<'a, B>) -> &'a B::Framebuffer
where
    B: Backend,
{
    use self::SuperFramebuffer::*;
    use self::SuperFrame::*;
    match (framebuffer, frame) {
        (&Owned(ref owned), _) => owned,
        (&Acquired(ref acquired), Index(index)) => &acquired[index],
        (&Single, Buffer(ref target)) => target,
        _ => unreachable!("This combination can't happen"),
    }
}

#[derive(Debug)]
pub struct PassNode<B: Backend> {
    clears: Vec<ClearValue>,
    descriptors: B::DescriptorSetLayout,
    layout: B::PipelineLayout,
    pipeline: B::GraphicsPipeline,
    render_pass: B::RenderPass,
    framebuffer: SuperFramebuffer<B>,
    pass: Box<AnyPass<B>>,
    depends: Vec<(usize, PipelineStage)>,
}

impl<B> PassNode<B>
where
    B: Backend,
{
    fn draw(
        &mut self,
        cbuf: &mut B::CommandBuffer,
        device: &mut B::Device,
        world: &World,
        rect: Rect,
        frame: SuperFrame<B>,
    ) -> bool {
        use gfx_hal::command::RawCommandBuffer;

        // Bind pipeline
        cbuf.bind_graphics_pipeline(&self.pipeline);

        // Run custom preparation
        // * Write descriptor sets
        // * Store caches
        self.pass.prepare(cbuf, &self.layout, device, world);

        // Begin render pass with single inline subpass
        cbuf.begin_renderpass(
            &self.render_pass,
            pick(&self.framebuffer, frame),
            rect,
            &self.clears, // TODO: Put clear values here
            SubpassContents::Inline,
        );
        // Record custom drawing calls
        self.pass.draw(cbuf, world);

        // End the only renderpass
        cbuf.end_renderpass();

        // Return if this pass renders to the acquired image
        self.framebuffer.is_acquired()
    }
}


#[derive(Debug)]
pub struct RenderGraph<B: Backend> {
    passes: Vec<PassNode<B>>,
    leafs: Vec<usize>,
    signals: Vec<B::Semaphore>,
    acquire: B::Semaphore,
    finish: B::Fence,
    backbuffer: Backbuffer<B>,
    images: Vec<B::Image>,
    views: Vec<B::ImageView>,
}

impl<B> RenderGraph<B>
where
    B: Backend,
{
    fn draw<S, C>(
        &mut self,
        pool: &mut B::CommandPool,
        cbufs: &mut Vec<B::CommandBuffer>,
        queue: &mut CommandQueue<B, C>,
        swapchain: &mut S,
        device: &mut B::Device,
        viewport: &Viewport,
        world: &World,
    ) where
        S: Swapchain<B>,
        C: Supports<Graphics> + Supports<Transfer>,
    {
        use std::iter::once;
        use gfx_hal::command::RawCommandBuffer;
        use gfx_hal::image::ImageLayout;
        use gfx_hal::pool::RawCommandPool;
        use gfx_hal::queue::RawCommandQueue;
        use gfx_hal::queue::submission::RawSubmission;
        use gfx_hal::window::FrameSync;

        let ref signals = self.signals;
        let ref finish = self.finish;
        let ref acquire = self.acquire;
        let count = self.passes.len();

        // Start frame acquisition
        let frame = SuperFrame::new(
            &self.backbuffer,
            swapchain.acquire_frame(FrameSync::Semaphore(acquire)),
        );



        // Allocate enough command buffers
        if cbufs.len() < self.passes.len() {
            let add = self.passes.len() - cbufs.len();
            cbufs.append(&mut pool.allocate(add));
        }

        // Store `Semaphore`s that `Surface::repsent` needs to wait for
        let mut presents: SmallVec<[_; 32]> = SmallVec::new();

        // Record commands for all passes
        self.passes.iter_mut().enumerate().for_each(|(id, pass)| {
            // Pick buffer
            let ref mut cbuf = cbufs[id];

            // Begin writing
            cbuf.begin();
            // Setup
            cbuf.set_viewports(&[viewport.clone()]);
            cbuf.set_scissors(&[viewport.rect]);

            // Record commands for pass
            let acquires = pass.draw(cbuf, device, world, viewport.rect, frame.clone());

            // If it renders to acquired image
            let wait_acuired = if acquires {
                // Presenting has to wait for it
                presents.push(&signals[id]);
                // And it should wait for acqusition
                Some((acquire, PipelineStage::COLOR_ATTACHMENT_OUTPUT))
            } else {
                None
            };

            // finish buffer recording
            cbuf.finish();
            unsafe {
                // Submit buffer
                queue.as_mut().submit_raw(
                    RawSubmission {
                        cmd_buffers: &[cbuf.clone()],
                        wait_semaphores: &pass.depends
                            .iter()
                            .map(|&(id, stage)| (&signals[id], stage))
                            .chain(wait_acuired)
                            .collect::<SmallVec<[_; 32]>>(),

                        // Each pass singnals to associated `Semaphore`
                        signal_semaphores: &[&signals[id]],
                    },
                    // Signal the fence in last submission
                    if id == count - 1 { Some(finish) } else { None },
                );
            }
        });

        // Present queue
        swapchain.present(queue, &presents);

        // Wait defice to finish the work
        device.wait_for_fences(&[finish], WaitFor::All, !0);

        // Rest command buffers
        pool.reset();
    }
}
