//! ECS rendering bundle

use amethyst_assets::Processor;
use amethyst_core::{
    bundle::{Result, ResultExt, SystemBundle},
    specs::prelude::DispatcherBuilder,
};

use crate::{
    config::DisplayConfig,
    pass::{Flat2DDataSorter, Flat2DImageEncoder, Flat2DSpriteEncoder, Flat2DSpriteSheetEncoder},
    pipe::{PipelineBuild, PolyPipeline},
    sprite::SpriteSheet,
    system::RenderSystem,
    visibility::VisibilitySortingSystem,
    HideHierarchySystem,
};

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
pub struct RenderBundle<'a, B, P>
where
    B: PipelineBuild<Pipeline = P>,
    P: PolyPipeline,
{
    pipe: B,
    config: Option<DisplayConfig>,
    visibility_sorting: Option<&'a [&'a str]>,
    drawflat2d_encoders_deps: Option<&'a [&'a str]>,
    drawflat2d_external_encoders: Option<&'a [&'a str]>,
    sprite_sheet_processor_enabled: bool,
    sprite_processor_enabled: bool,
    hide_hierarchy_system_enabled: bool,
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
            drawflat2d_encoders_deps: None,
            drawflat2d_external_encoders: None,
            sprite_sheet_processor_enabled: false,
            sprite_processor_enabled: false,
            hide_hierarchy_system_enabled: false,
        }
    }

    /// Enable transparent mesh sorting, with the given dependencies
    pub fn with_visibility_sorting(mut self, dep: &'a [&'a str]) -> Self {
        self.visibility_sorting = Some(dep);
        self
    }

    /// Enable encoders for `DrawFlat2D` render pass.
    ///
    /// This has to be enabled for the pass to render anything on the screen.
    pub fn with_drawflat2d_encoders(mut self, dep: &'a [&'a str]) -> Self {
        self.drawflat2d_encoders_deps = Some(dep);
        self
    }

    /// Register additional encoders for `DrawFlat2D` render pass.
    ///
    /// Requires `with_drawflat2d_encoders` to be configured.
    ///
    /// Note that this only sets up the system dependencies.
    /// You still have to register the systems on your own.
    pub fn with_external_drawflat2d_encoders(mut self, dep: &'a [&'a str]) -> Self {
        self.drawflat2d_external_encoders = Some(dep);
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

    /// Enable the sprite sheet processor
    ///
    /// If you load a `Sprite` in memory as an asset `Format`, this adds the `Processor` that
    /// will convert it to the `Asset`.
    pub fn with_sprite_processor(mut self) -> Self {
        self.sprite_processor_enabled = true;
        self
    }

    /// Enable the [hierarchical hiding system](struct.HideHierarchySystem.html).
    /// Requires the `"parent_hierarchy_system"` to be used, which is a default part of TransformBundle.
    pub fn with_hide_hierarchy_system(mut self) -> Self {
        self.hide_hierarchy_system_enabled = true;
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

        // TODO: Extract DrawFlat2D specific settings, possibly to a separate bundle.
        if let Some(deps) = self.drawflat2d_encoders_deps {
            builder.add(Flat2DImageEncoder::default(), "flat2d_image_encoder", deps);
            builder.add(
                Flat2DSpriteEncoder::default(),
                "flat2d_sprite_encoder",
                deps,
            );
            builder.add(
                Flat2DSpriteSheetEncoder::default(),
                "flat2d_spritesheet_encoder",
                deps,
            );

            let mut encoders = vec![
                "flat2d_image_encoder",
                "flat2d_sprite_encoder",
                "flat2d_spritesheet_encoder",
            ];

            if let Some(externals) = self.drawflat2d_external_encoders {
                encoders.extend(externals);
            };

            builder.add(Flat2DDataSorter::default(), "flat2d_data_sorter", &encoders);
        } else if self.drawflat2d_external_encoders.is_some() {
            return Err("You must use `with_drawflat2d_encoders` in order to make `with_external_drawflat2d_encoders` work".into());
        }

        builder.add_thread_local(
            RenderSystem::build(self.pipe, self.config).chain_err(|| "Renderer error!")?,
        );
        Ok(())
    }
}
