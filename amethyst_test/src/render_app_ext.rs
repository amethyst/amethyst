use amethyst::{
    assets::Processor,
    core::TransformBundle,
    ecs::Resources,
    renderer::{
        rendy::{factory::Factory, graph::GraphBuilder},
        sprite::SpriteSheet,
        types::DefaultBackend,
        GraphCreator, RenderingSystem,
    },
    window::ScreenDimensions,
    GameData, StateEvent, StateEventReader,
};

use crate::{AmethystApplication, GameUpdate, HIDPI, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Extension to include render specific functions.
pub trait RenderBaseAppExt {
    /// Provides base bundles and systems for an application with render functionality.
    fn render_base() -> Self;
}

impl RenderBaseAppExt
    for AmethystApplication<GameData<'static, 'static>, StateEvent, StateEventReader>
{
    fn render_base() -> Self {
        AmethystApplication::blank()
            .with_rendering_system()
            .with_bundle(TransformBundle::new())
            .with_system(
                Processor::<SpriteSheet>::new(),
                "sprite_sheet_processor",
                &[],
            )
            .with_resource(ScreenDimensions::new(SCREEN_WIDTH, SCREEN_HEIGHT, HIDPI))
    }
}

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
