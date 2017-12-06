use std::fmt;

use gfx_hal::Backend;
use gfx_hal::command::{Rect, Viewport};
use gfx_hal::device::Extent;
use gfx_hal::format::Format;
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::{CommandQueue, Graphics, Supports, Transfer};
use gfx_hal::window::{Backbuffer, Surface};

use specs::World;

use winit::{EventsLoop, Window};

use command::{CommandCenter, Execution};
use epoch::{CurrentEpoch, Epoch};
use graph::Graph;
use graph::build::ColorPin;
use memory::Allocator;
use shaders::ShaderManager;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RendererBuilder<'a> {
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
    #[derivative(Debug = "ignore")]
    pub surface: B::Surface,
    pub format: Format,
    #[derivative(Debug = "ignore")]
    pub swapchain: B::Swapchain,
    pub backbuffer: Backbuffer<B>,
    pub shaders: ShaderManager<B>,
    pub graphs: Vec<(Graph<B>, Epoch)>,
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
        device: &B::Device,
        world: &World,
    ) {
        let start = self.graphs[graph].1;
        let span = self.graphs[graph].0.get_frames_number();
        center.execute_graphics(
            Draw {
                renderer: self,
                world,
                graph,
            },
            start,
            span,
            current,
            device,
        );
        self.graphs[graph].1 = current.now() + 1;
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

        self.graphs.push((graph, Epoch::new()));
        Ok(self.graphs.len() - 1)
    }
}

struct Draw<'a, B: Backend> {
    renderer: &'a mut Renderer<B>,
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
        fence: &B::Fence,
        device: &B::Device,
    ) {
        let ref mut graph = self.renderer.graphs[self.graph].0;
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
        graph.draw(
            queue,
            &mut pools[0],
            swapchain,
            backbuffer,
            device,
            viewport,
            world,
            fence,
        );
    }
}
