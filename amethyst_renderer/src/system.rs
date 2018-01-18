//! Rendering system
//! 
//! 
//! 

use std::marker::PhantomData;

use gfx_hal::Backend;

use shred::{ResourceId, Resources, RunNow};
use specs::{System, SystemData, World};

use command::CommandCenter;
use epoch::CurrentEpoch;
use memory::Allocator;
use hal::{Hal, Renderer};
use upload::Uploader;


pub struct ActiveGraph(pub usize);

pub struct AllResources<'a>(&'a Resources);
impl<'a> SystemData<'a> for AllResources<'a> {
    fn fetch(res: &'a Resources, _id: usize) -> Self { AllResources(res) }
    fn reads(id: usize) -> Vec<ResourceId> { vec![] }
    fn writes(id: usize) -> Vec<ResourceId> { vec![] }
}

pub struct RenderingSystem<B: Backend> {
    pub center: CommandCenter<B>,
    pub renderer: Option<Renderer<B>>,
}

impl<'a, B> System<'a> for RenderingSystem<B>
where
    B: Backend,
{
    type SystemData = AllResources<'a>;
    fn run(&mut self, AllResources(res): AllResources<'a>) {
        let graph = res.try_fetch::<ActiveGraph>(0).map(|ag| ag.0).unwrap_or(0);
        res.fetch_mut::<Hal<B>>(0);
    }
}

impl<B> RenderingSystem<B>
where
    B: Backend,
{
    pub fn cleanup(&mut self, res: &Resources) {
        let Hal { ref device, ref mut allocator, ref mut current, .. } = *res.fetch_mut::<Hal<B>>(0);
        self.center.wait_finish(device, current);
        self.renderer.take().map(|renderer| {
            renderer.dispose(allocator, device, res)
        });
    }
}

