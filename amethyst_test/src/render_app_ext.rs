use amethyst::{
    assets::Processor,
    core::TransformBundle,
    ecs::Resources,
    renderer::{
        pass::{DrawFlat2DDesc, DrawFlat2DTransparentDesc},
        rendy::{
            factory::Factory,
            graph::{
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::{
                command::{ClearDepthStencil, ClearValue},
                format::Format,
                image::Kind,
            },
        },
        sprite::SpriteSheet,
        types::DefaultBackend,
        GraphCreator, RenderingSystem,
    },
    ui::DrawUiDesc,
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
        let window_kind = Kind::D2(SCREEN_WIDTH, SCREEN_HEIGHT, 1, 1);
        let surface_format = Format::Rgba32Sfloat; // Normally extracted from the `Window`.

        let mut graph_builder = GraphBuilder::new();
        let colour = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0., 0., 0., 1.].into())),
        );

        // Depth stencil must be 1. for the background to be drawn.
        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1., 0))),
        );

        let _sprite = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DDesc::new().builder())
                .with_color(colour)
                .with_depth_stencil(depth)
                .into_pass(),
        );
        let _sprite_trans = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DTransparentDesc::new().builder())
                .with_color(colour)
                .with_depth_stencil(depth)
                .into_pass(),
        );
        let _ui = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawUiDesc::new().builder())
                .with_color(colour)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        graph_builder
    }
}
