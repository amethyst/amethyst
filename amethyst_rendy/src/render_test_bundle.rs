use std::{ops::Deref, sync::Arc};

use amethyst_assets::Processor;
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, ReadExpect, Resources, SystemData},
};
use amethyst_error::Error;
use amethyst_window::{ScreenDimensions, Window};
use derive_new::new;

use crate::{
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
    sprite_visibility::SpriteVisibilitySortingSystem,
    types::DefaultBackend,
    GraphCreator, RenderingSystem,
};

/// Adds sprite systems and a basic rendering system to the dispatcher.
///
/// This test bundle requires the user to also add the `TransformBundle` and `WindowBundle` to the
/// dispatcher.
#[derive(Debug, new)]
pub struct RenderTestBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for RenderTestBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        );
        builder.add(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &["transform_system"],
        );

        builder.add_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            RenderGraph::default(),
        ));

        Ok(())
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

        graph_builder
    }
}
