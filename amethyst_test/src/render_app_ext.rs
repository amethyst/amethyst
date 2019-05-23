use std::{ops::Deref, sync::Arc};

use amethyst::{
    assets::Processor,
    core::TransformBundle,
    ecs::{ReadExpect, Resources, SystemData},
    renderer::{
        pass::{DrawFlat2DDesc, DrawFlat2DTransparentDesc},
        rendy::{
            factory::Factory,
            graph::{
                present::PresentNode,
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
        sprite_visibility::SpriteVisibilitySortingSystem,
        types::DefaultBackend,
        GraphCreator, RenderingSystem,
    },
    window::{DisplayConfig, ScreenDimensions, Window, WindowBundle},
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
        let mut display_config = DisplayConfig::default();
        display_config.dimensions = Some((SCREEN_WIDTH, SCREEN_HEIGHT));
        display_config.visibility = false;

        AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(WindowBundle::from_config(display_config))
            .with_rendering_system()
            .with_system(
                Processor::<SpriteSheet>::new(),
                "sprite_sheet_processor",
                &[],
            )
            .with_system(
                SpriteVisibilitySortingSystem::new(),
                "sprite_visibility_system",
                &["transform_system"],
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
            RenderGraph::default(),
        ))
    }
}

/// Default render graph in case the `RenderingSystem` is only needed to load textures and meshes.
#[derive(Default)]
pub struct RenderGraph {
    dimensions: Option<ScreenDimensions>,
    surface_format: Option<Format>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for RenderGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        self.dirty = false;

        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);
        let surface = factory.create_surface(&window);
        // cache surface format to speed things up
        let surface_format = *self
            .surface_format
            .get_or_insert_with(|| factory.get_surface_format(&surface));
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

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

        let sprite = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DDesc::new().builder())
                .with_color(colour)
                .with_depth_stencil(depth)
                .into_pass(),
        );
        let sprite_trans = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DTransparentDesc::new().builder())
                .with_color(colour)
                .with_depth_stencil(depth)
                .into_pass(),
        );
        let _present = graph_builder.add_node(
            PresentNode::builder(factory, surface, colour)
                .with_dependency(sprite_trans)
                .with_dependency(sprite),
        );

        graph_builder
    }
}
