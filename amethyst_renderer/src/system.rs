//! Rendering system
//! 
//! 
//! 

use std::marker::PhantomData;

use gfx_hal::Backend;

use shred::{Resources, RunNow};

use hal::Hal;


struct RenderingSystem<B: Backend> {
    pub center: CommandCenter<B>,
    pub shaders: ShaderManager<B>,
}

impl<B> RenderingSystem<B>
where
    B: Backend,
{
    fn new(hal: Hal<B>, world: &mut World) -> Self {
        let Hal {
            device,
            allocator,
            center,
            uploader,
            renderer,
            current,
            shaders,
            ..
        } = hal;

        RenderingSystem {
            center,
            shaders,
        }
    }
}

struct HalResource<B: Backend> {
    pub device: B::Device,
    pub allocator: Allocator<B>,
    pub uploader: Uploader<B>,
    pub renderer: Option<Renderer<B>>,
    pub current: CurrentEpoch,
    pub shaders: ShaderManager<B>,
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


