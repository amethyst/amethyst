//! ECS rendering bundle

use amethyst_assets::Processor;
use amethyst_core::{bundle::SystemBundle, ecs::prelude::DispatcherBuilder, math::RealField};
use amethyst_error::{format_err, Error, ResultExt};
use std::marker::PhantomData;

use crate::{
    config::DisplayConfig,
    pipe::{PipelineBuild, PolyPipeline},
    sprite::SpriteSheet,
    sprite_visibility::SpriteVisibilitySortingSystem,
    system::RenderSystem,
    visibility::VisibilitySortingSystem,
    HideHierarchySystem,
};

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
/// Will register `TransparentSortingSystem`, with name `transparent_sorting_system` if sorting is
/// requested.
///
/// # Type Parameters:
///
/// * `N`: `RealBound` (f32, f64)
pub struct RenderBundle<'a, B, P, N>
where
    B: PipelineBuild<Pipeline = P>,
    P: PolyPipeline,
    N: RealField,
{
    pipe: B,
    config: Option<DisplayConfig>,
    visibility_sorting: Option<&'a [&'a str]>,
    sprite_visibility_sorting: Option<&'a [&'a str]>,
    sprite_sheet_processor_enabled: bool,
    hide_hierarchy_system_enabled: bool,
    _pd: PhantomData<N>,
}

impl<'a, B, P, N> RenderBundle<'a, B, P, N>
where
    B: PipelineBuild<Pipeline = P>,
    P: PolyPipeline,
    N: RealField,
{
    /// Create a new render bundle
    pub fn new(pipe: B, config: Option<DisplayConfig>) -> Self {
        RenderBundle {
            pipe,
            config,
            visibility_sorting: None,
            sprite_visibility_sorting: None,
            sprite_sheet_processor_enabled: false,
            hide_hierarchy_system_enabled: false,
            _pd: PhantomData,
        }
    }

    /// Enable transparent mesh sorting, with the given dependencies
    pub fn with_visibility_sorting(mut self, dep: &'a [&'a str]) -> Self {
        self.visibility_sorting = Some(dep);
        self
    }

    /// Enable transparent sprite sorting, with the given dependencies
    pub fn with_sprite_visibility_sorting(mut self, dep: &'a [&'a str]) -> Self {
        self.sprite_visibility_sorting = Some(dep);
        self
    }

    /// Enable the sprite sheet processor
    ///
    /// If you load a `SpriteSheet` in memory as an asset `Format`, this adds the `Processor` that
    /// will convert it to the `Asset`.
    pub fn with_sprite_sheet_processor(mut self) -> Self {
        self.sprite_sheet_processor_enabled = true;
        self
    }

    /// Enable the [hierarchical hiding system](struct.HideHierarchySystem.html).
    /// Requires the `"parent_hierarchy_system"` to be used, which is a default part of TransformBundle.
    pub fn with_hide_hierarchy_system(mut self) -> Self {
        self.hide_hierarchy_system_enabled = true;
        self
    }
}

impl<'a, 'b, 'c, B, P, N> SystemBundle<'a, 'b> for RenderBundle<'c, B, P, N>
where
    B: PipelineBuild<Pipeline = P>,
    P: 'b + PolyPipeline,
    N: RealField + Default,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        if let Some(dep) = self.visibility_sorting {
            builder.add(
                VisibilitySortingSystem::<N>::new(),
                "visibility_sorting_system",
                dep,
            );
        };
        if let Some(dep) = self.sprite_visibility_sorting {
            builder.add(
                SpriteVisibilitySortingSystem::<N>::new(),
                "sprite_visibility_sorting_system",
                dep,
            );
        };
        if self.sprite_sheet_processor_enabled {
            builder.add(
                Processor::<SpriteSheet>::new(),
                "sprite_sheet_processor",
                &[],
            );
        }
        if self.hide_hierarchy_system_enabled {
            builder.add(
                HideHierarchySystem::default(),
                "hide_hierarchy_system",
                &["parent_hierarchy_system"],
            );
        }
        builder.add_thread_local(
            RenderSystem::build(self.pipe, self.config)
                .with_context(|_| format_err!("Renderer error!"))?,
        );
        Ok(())
    }
}
