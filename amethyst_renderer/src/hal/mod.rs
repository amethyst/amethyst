pub mod build;

use std::mem::ManuallyDrop;
use std::ptr::read;

use gfx_hal::Backend;

use command::CommandCenter;
use epoch::CurrentEpoch;
use memory::Allocator;
use renderer::Renderer;
use shaders::ShaderManager;
use upload::Uploader;

pub use self::build::{Error, ErrorKind, HalConfig};

pub struct Hal<B: Backend> {
    pub device: B::Device,
    pub allocator: Allocator<B>,
    pub center: CommandCenter<B>,
    pub uploader: ManuallyDrop<Uploader<B>>,
    pub renderer: Option<Renderer<B>>,
    pub current: CurrentEpoch,
    pub shaders: ShaderManager<B>,
}


impl<B> Drop for Hal<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        self.center.wait_finish(&self.device, &mut self.current);
        unsafe {
            ManuallyDrop::into_inner(read(&mut self.uploader)).dispose(&mut self.allocator);
        }
        self.renderer.take().map(|renderer| {
            renderer.dispose(&mut self.allocator, &self.device)
        });
        self.shaders.unload(&self.device);
        self.allocator.cleanup(&self.device, &self.current);
    }
}
