//! Encapsulation for external rendering resources.
//! 

use std::fmt;

use gfx_hal::{Backend, Device};
use gfx_hal::command::{Rect, Viewport};
use gfx_hal::device::Extent;
use gfx_hal::format::Format;
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::{CommandQueue, Graphics, Supports, Transfer, Submission};
use gfx_hal::window::{Backbuffer, Swapchain};

use specs::World;

use winit::{EventsLoop, Window};

use cirque::Cirque;
use command::{CommandCenter, Execution};
use epoch::{CurrentEpoch, Epoch};
use graph::{Graph, SuperFrame};
use graph::ColorPin;
use memory::Allocator;
use shaders::ShaderManager;
use upload::Uploader;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RendererConfig<'a> {
    pub title: &'a str,
    pub width: u16,
    pub height: u16,

    #[derivative(Debug = "ignore")]
    pub events: &'a EventsLoop,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Renderer<B: Backend> {
    #[derivative(Debug(format_with = "fmt_window"))]
    pub window: Window,
    pub format: Format,

    #[derivative(Debug = "ignore")]
    pub surface: B::Surface,
    #[derivative(Debug = "ignore")]
    pub swapchain: B::Swapchain,
    pub backbuffer: Backbuffer<B>,
    pub surface_semaphores: Cirque<(B::Semaphore, B::Semaphore)>,
    pub start_epoch: Epoch,
    pub graphs: Vec<Graph<B>>, // Move it to `Hal`.
}


fn fmt_window(window: &Window, fmt: &mut fmt::Formatter) -> fmt::Result {
    write!(fmt, "Window({:?})", window.id())
}

impl<B> Renderer<B>
where
    B: Backend,
{
    pub fn draw(
        &mut self,
        graph: usize,
        current: &mut CurrentEpoch,
        center: &mut CommandCenter<B>,
        allocator: &mut Allocator<B>,
        uploader: Option<&mut Uploader<B>>,
        device: &B::Device,
        world: &World,
    ) {
        let start_epoch = self.start_epoch;
        center.execute_graphics(
            Draw {
                allocator,
                renderer: self,
                world,
                graph,
                uploader,
            },
            start_epoch,
            current,
            device,
        );
        self.start_epoch = current.now() + 1;
    }

    pub fn add_graph(
        &mut self,
        present: ColorPin<B>,
        device: &B::Device,
        allocator: &mut Allocator<B>,
        shaders: &mut ShaderManager<B>,
    ) -> ::graph::Result<usize> {
        let (width, height) = self.window.get_inner_size_pixels().unwrap();
        let graph = Graph::build(
            present,
            &self.backbuffer,
            self.format,
            None,
            Extent {
                width,
                height,
                depth: 1,
            },
            device,
            allocator,
            shaders,
        )?;

        self.graphs.push(graph);
        Ok(self.graphs.len() - 1)
    }

    pub fn dispose(mut self, allocator: &mut Allocator<B>, device: &B::Device) {
        for graph in self.graphs {
            graph.dispose(allocator, device);
        }
        for (_, (acq, rel)) in unsafe { self.surface_semaphores.take() } {
            device.destroy_semaphore(acq);
            device.destroy_semaphore(rel);
        }
    }

    fn get_frames_number(&self) -> usize {
        match self.backbuffer {
            Backbuffer::Images(ref images) => images.len(),
            Backbuffer::Framebuffer(_) => 1,
        }
    }
}

struct Draw<'a, B: Backend> {
    allocator: &'a mut Allocator<B>,
    renderer: &'a mut Renderer<B>,
    uploader: Option<&'a mut Uploader<B>>,
    world: &'a World,
    graph: usize,
}

impl<'a, B, C> Execution<B, C> for Draw<'a, B>
where
    B: Backend,
    C: Supports<Graphics> + Supports<Transfer>,
{
    fn execute(
        self,
        queue: &mut CommandQueue<B, C>,
        pools: &mut [CommandPool<B, C>],
        current: &CurrentEpoch,
        fence: &B::Fence,
        device: &B::Device,
    ) -> Epoch {
        use gfx_hal::window::FrameSync;

        let frames = self.renderer.get_frames_number();
        let ref mut graph = self.renderer.graphs[self.graph];
        let ref mut swapchain = self.renderer.swapchain;
        let ref backbuffer = self.renderer.backbuffer;
        let (width, height) = self.renderer.window.get_inner_size_pixels().unwrap();
        let viewport = Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: width as _,
                h: height as _,
            },
            depth: 0.0..1.0,
        };

        let ref world = *self.world;


        // This execuiton needs to finsish before we will draw same frame again.
        // If there is only one frame this execution have to finish in current epoch
        let finish = current.now() + (frames - 1) as u64;
        let span = current.now() .. finish;

        let acq_rel = self.renderer.surface_semaphores.get_or_insert(span.clone(), || (
            device.create_semaphore(),
            device.create_semaphore(),
        ));

        // Start frame acquisition
        let frame = SuperFrame::new(
            backbuffer,
            swapchain.acquire_frame(FrameSync::Semaphore(&acq_rel.0)),
        );

        let upload = self.uploader.and_then(|uploader| uploader.commit(span.clone(), device, queue, &mut pools[0]));

        graph.draw_inline(
            queue,
            &mut pools[0],
            frame,
            upload.as_ref().map(|x|&**x),
            &acq_rel.0,
            &acq_rel.1,
            self.allocator,
            device,
            viewport,
            world,
            fence,
            span,
        );

        // Present queue
        profile_scope!("Graph::draw :: present");
        swapchain.present(queue, &[&acq_rel.1]);
        finish
    }
}
