//! Rendering system
//!
//!
//!

use std::marker::PhantomData;

use hal::Backend;

use shred::{ResourceId, Resources, RunNow};
use specs::{System, SystemData, World};

use command::CommandCenter;
use epoch::CurrentEpoch;
use hal::{BasicFactory, Renderer};
use memory::Allocator;
use upload::Uploader;

pub struct ActiveGraph(pub Option<usize>);

pub struct AllResources<'a>(&'a Resources);
impl<'a> SystemData<'a> for AllResources<'a> {
    fn fetch(res: &'a Resources, _id: usize) -> Self {
        AllResources(res)
    }
    fn reads(id: usize) -> Vec<ResourceId> {
        vec![]
    }
    fn writes(id: usize) -> Vec<ResourceId> {
        vec![]
    }
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
        if let Some(graph) = res.fetch::<ActiveGraph>(0).0 {
            if let Some(ref mut renderer) = self.renderer {
                let hal = &mut *res.fetch_mut::<BasicFactory<B>>(0);

                renderer.draw(
                    graph,
                    &mut hal.current,
                    &mut self.center,
                    &mut hal.allocator,
                    Some(&mut hal.uploader),
                    &mut hal.device,
                    res,
                );
            }
        }
    }
}

impl<B> RenderingSystem<B>
where
    B: Backend,
{
    pub fn cleanup(&mut self, res: &Resources) {
        let BasicFactory {
            ref device,
            ref mut allocator,
            ref mut current,
            ..
        } = *res.fetch_mut::<BasicFactory<B>>(0);
        self.center.wait_finish(device, current);
        self.renderer
            .take()
            .map(|renderer| renderer.dispose(allocator, device, res));
    }
}
