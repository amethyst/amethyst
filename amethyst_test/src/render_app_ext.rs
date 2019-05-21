use amethyst::{
    ecs::Resources,
    renderer::{
        rendy::{factory::Factory, graph::GraphBuilder},
        types::DefaultBackend,
        GraphCreator, RenderingSystem,
    },
};

use crate::{AmethystApplication, GameUpdate};

/// Extension to include render specific functions.
pub trait RenderAppExt {
    /// Adds a `RenderingSystem` with an empty graph.
    fn with_rendering_system(self) -> Self;
}

impl<T, E, R> RenderAppExt for AmethystApplication<T, E, R>
where
    T: GameUpdate,
    E: Send + Sync + 'static,
{
    fn with_rendering_system(self) -> Self {
        self.with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            RenderGraphEmpty::default(),
        ))
        .run_in_thread()
    }
}

/// Empty render graph in case the `RenderingSystem` is only needed to load textures and meshes.
#[derive(Default)]
pub struct RenderGraphEmpty;

impl GraphCreator<DefaultBackend> for RenderGraphEmpty {
    fn rebuild(&mut self, _res: &Resources) -> bool {
        false
    }

    fn builder(
        &mut self,
        _factory: &mut Factory<DefaultBackend>,
        _res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        GraphBuilder::new()
    }
}
