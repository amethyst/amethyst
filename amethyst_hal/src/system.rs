use hal::Backend;
use shred::{Resources, RunNow};
use std::marker::PhantomData;
use xfg::Pass;

use factory::Factory;
use renderer::Renderer;

///
pub struct RenderSystem<B, P>(PhantomData<fn() -> (B, P)>);
impl<B, P> RenderSystem<B, P> {
    pub fn new() -> Self {
        RenderSystem(PhantomData)
    }
}

impl<'a, B, P> RunNow<'a> for RenderSystem<B, P>
where
    B: Backend,
    P: Pass<B, &'a Resources> + Send + Sync + 'static,
{
    fn run_now(&mut self, res: &'a Resources) {
        let ref mut factory = *res.fetch_mut::<Factory<B>>(0);
        let ref mut renderer = *res.fetch_mut::<Renderer<B, P>>(0);
        renderer.run(res, factory);
    }
}
