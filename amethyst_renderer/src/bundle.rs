//! ECS rendering bundle

use amethyst_assets::Processor;
use amethyst_core::bundle::{Result, ResultExt, SystemBundle};
use amethyst_core::specs::prelude::DispatcherBuilder;
use config::DisplayConfig;
use pipe::{PipelineBuild, PolyPipeline};
use sprite::SpriteSheet;
use system::RenderSystem;
use visibility::VisibilitySortingSystem;

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
/// Will register `TransparentSortingSystem`, with name `transparent_sorting_system` if sorting is
/// requested.
///
pub struct RenderBundle<'a, B, P>
where
    B: PipelineBuild<Pipeline = P>,
    P: PolyPipeline,
{
    pipe: B,
    config: Option<DisplayConfig>,
    visibility_sorting: Option<&'a [&'a str]>,
}

impl<'a, B, P> RenderBundle<'a, B, P>
where
    B: PipelineBuild<Pipeline = P>,
    P: PolyPipeline,
{
    /// Create a new render bundle
    pub fn new(pipe: B, config: Option<DisplayConfig>) -> Self {
        RenderBundle {
            pipe,
            config,
            visibility_sorting: None,
        }
    }

    /// Enable transparent mesh sorting, with the given dependencies
    pub fn with_visibility_sorting(mut self, dep: &'a [&'a str]) -> Self {
        self.visibility_sorting = Some(dep);
        self
    }
}

impl<'a, 'b, 'c, B: PipelineBuild<Pipeline = P>, P: 'b + PolyPipeline> SystemBundle<'a, 'b>
    for RenderBundle<'c, B, P>
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        if let Some(dep) = self.visibility_sorting {
            builder.add(
                VisibilitySortingSystem::new(),
                "visibility_sorting_system",
                dep,
            );
        };
        builder.add_thread_local(
            RenderSystem::build(self.pipe, self.config).chain_err(|| "Renderer error!")?,
        );
        builder.add(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        );
        Ok(())
    }
}
