use std::cmp::min;

use gfx_hal::{Backend, Device, Gpu, Instance};
use gfx_hal::adapter::{Adapter, PhysicalDevice};
use gfx_hal::format::{ChannelType, Format, Formatted, Srgba8};
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::{CommandQueue, Compute, General, Graphics, QueueFamily, QueueGroup, QueueType,
                     RawQueueGroup, Transfer};
use gfx_hal::window::{Backbuffer, Surface, SwapchainConfig};

use winit::{EventsLoop, Window, WindowBuilder};


use command::CommandCenter;
use epoch::CurrentEpoch;
use graph::{Graph, Present};
use memory::Allocator;
use shaders::{ShaderLoader, ShaderManager};
use upload::Uploader;


pub mod build;



pub struct Hal<B: Backend> {
    pub device: B::Device,
    pub allocator: Allocator<B>,
    pub center: CommandCenter<B>,
    pub uploader: Uploader<B>,
    pub renderer: Option<Renderer<B>>,
    pub current_epoch: CurrentEpoch,
}


pub struct Renderer<B: Backend> {
    window: Window,
    surface: B::Surface,
    format: Format,
    swapchain: B::Swapchain,
    backbuffer: Backbuffer<B>,
    shaders: ShaderManager<B>,
    graphs: Vec<Graph<B>>,
}
