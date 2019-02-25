//! Renderer system
use {
    amethyst_core::specs::{Resources, RunNow},
    rendy::{command::Families, factory::Factory, graph::Graph, hal::Backend},
};

pub struct RendererSystem<B: Backend> {
    graph: Graph<B, Resources>,
}

impl<'a, B> RunNow<'a> for RendererSystem<B>
where
    B: Backend,
{
    fn run_now(&mut self, res: &'a Resources) {
        let mut factory = res.fetch_mut::<Factory<B>>();
        let mut families = res.fetch_mut::<Families<B>>();
        self.graph.run(&mut factory, &mut families, res);
    }

    fn setup(&mut self, _res: &mut Resources) {}
}
