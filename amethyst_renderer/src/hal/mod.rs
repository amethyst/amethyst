//!
//! Top-level structure that encapsulates all pieces of rendering engine.
//! `HalConfig` to instantiate `Hal`.
//!

mod build;
mod renderer;

use std::mem::ManuallyDrop;
use std::ptr::read;

use gfx_hal::Backend;

use shred::Resources;

use command::CommandCenter;
use epoch::CurrentEpoch;
use memory::Allocator;
use relevant::Relevant;
use upload::Uploader;

pub use hal::build::{Error, ErrorKind, HalConfig};
pub use hal::renderer::{Renderer, RendererConfig};

pub struct Hal<B: Backend> {
    pub device: B::Device,
    pub allocator: Allocator<B>,
    pub center: CommandCenter<B>,
    pub uploader: Uploader<B>,
    pub renderer: Option<Renderer<B>>,
    pub current: CurrentEpoch,
    relevant: Relevant,
}

impl<B> Hal<B>
where
    B: Backend,
{
    pub fn dispose(mut self, res: &Resources) {
        self.center.wait_finish(&self.device, &mut self.current);
        self.renderer.take().map(|renderer| {
            renderer.dispose(&mut self.allocator, &self.device, res)
        });
        self.uploader.dispose(&mut self.allocator);
        self.allocator.cleanup(&self.device, &self.current);
    }
}
