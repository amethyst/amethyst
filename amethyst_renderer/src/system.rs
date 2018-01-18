//! Rendering system
//! 
//! 
//! 

use std::marker::PhantomData;

use gfx_hal::Backend;

use shred::{Resources, RunNow};
use specs::World;

use command::CommandCenter;
use epoch::CurrentEpoch;
use memory::Allocator;
use hal::{Hal, Renderer};
use upload::Uploader;


struct RenderingSystem<B: Backend> {
    pub center: CommandCenter<B>,
    pub renderer: Option<Renderer<B>>,
}

impl<B> Hal<B>
where
    B: Backend,
{
    fn into_system(self, world: &mut World) -> RenderingSystem<B> {
        let Hal {
            device,
            allocator,
            center,
            uploader,
            renderer,
            current,
            ..
        } = self;

        world.add_resource(HalResource {
            device: Device(device),
            allocator,
            uploader,
            current,
        });

        RenderingSystem {
            center,
            renderer,
        }
    }
}


/// `Backend::Device` are actually `Send + Sync`. Except for OpenGL.
pub struct Device<B: Backend>(B::Device);
unsafe impl<B> Send for Device<B> where B: Backend {}
unsafe impl<B> Sync for Device<B> where B: Backend {}

struct HalResource<B: Backend> {
    pub device: Device<B>,
    pub allocator: Allocator<B>,
    pub uploader: Uploader<B>,
    pub current: CurrentEpoch,
}

struct ActiveGraph(usize);

impl<'a, B> RunNow<'a> for RenderingSystem<B>
where
    B: Backend,
{
    fn run_now(&mut self, res: &'a Resources) {
        let graph = res.try_fetch::<ActiveGraph>(0).map(|ag| ag.0).unwrap_or(0);

        res.fetch_mut::<HalResource<B>>(0);
    }
}


