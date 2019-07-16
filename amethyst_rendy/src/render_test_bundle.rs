use std::{marker::PhantomData, ops::Deref, sync::Arc};

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
    types::Backend,
    GraphCreator, RenderingSystem,
};

/// Adds sprite systems and a basic rendering system to the dispatcher.
///
/// This test bundle requires the user to also add the `TransformBundle` and `WindowBundle` to the
/// dispatcher.
///
/// This is only meant for testing and only provides very basic rendering. You need to enable the
/// `test-support` flag to use this.
#[derive(Debug, new)]
pub struct RenderTestBundle<B>(PhantomData<B>);

impl<'a, 'b, B> SystemBundle<'a, 'b> for RenderTestBundle<B>
where
    B: Backend,
{
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

        builder.add_thread_local(RenderingSystem::<B, _>::new(RenderGraph::<B>::new()));

        Ok(())
    }
}

/// Adds sprite systems and a basic rendering system to the dispatcher.
///
/// This test bundle requires the user to also add the `TransformBundle` to the dispatcher.
///
/// This is only meant for automated testing and doesn't render anything. You need to enable the
///// `test-support` flag to use this.
#[derive(Debug, new)]
pub struct RenderEmptyBundle<B>(PhantomData<B>);

impl<'a, 'b, B> SystemBundle<'a, 'b> for RenderEmptyBundle<B>
where
    B: Backend,
{
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

        builder.add_thread_local(RenderingSystem::<B, _>::new(EmptyGraph::<B>::new()));

        Ok(())
    }
}

/// Render graph that renders sprites to a Window.
#[derive(Default, new)]
pub struct RenderGraph<B> {
    #[new(default)]
    dimensions: Option<ScreenDimensions>,
    #[new(default)]
    surface_format: Option<Format>,
    #[new(default)]
    dirty: bool,
    backend: PhantomData<B>,
}

impl<B> GraphCreator<B> for RenderGraph<B>
where
    B: Backend,
{
    #[allow(clippy::map_clone)]
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        self.dirty
    }

    fn builder(&mut self, factory: &mut Factory<B>, res: &Resources) -> GraphBuilder<B, Resources> {
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

/// Default render graph in case the `RenderingSystem` is only needed to load textures and meshes.
#[derive(Default, new)]
pub struct EmptyGraph<B>(PhantomData<B>);

impl<B> GraphCreator<B> for EmptyGraph<B>
where
    B: Backend,
{
    fn rebuild(&mut self, _res: &Resources) -> bool {
        false
    }

    fn builder(
        &mut self,
        _factory: &mut Factory<B>,
        _res: &Resources,
    ) -> GraphBuilder<B, Resources> {
        GraphBuilder::new()
    }
}
