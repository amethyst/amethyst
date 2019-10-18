//! Set of predefined implementations of `RenderPlugin` for use with `RenderingBundle`.

use crate::{
    legion::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pass::*,
        sprite_visibility::SpriteVisibilitySortingSystemDesc,
        visibility::VisibilitySortingSystemDesc,
    },
    Backend, Factory,
};
use amethyst_core::legion::{
    dispatcher::{DispatcherBuilder, Stage},
    SystemBundle, World,
};
use amethyst_error::Error;
use palette::Srgb;
use rendy::graph::render::RenderGroupDesc;

#[cfg(feature = "window")]
pub use window::RenderToWindow;

#[cfg(feature = "window")]
mod window {
    use super::*;
    use crate::{
        legion::bundle::{ImageOptions, OutputColor, TargetPlanOutputs},
        Format, Kind,
    };
    use amethyst_config::Config;

    use amethyst_window::{legion::WindowBundle, DisplayConfig, ScreenDimensions, Window};
    use rendy::hal::command::{ClearColor, ClearDepthStencil, ClearValue};
    use std::path::Path;

    /// A [RenderPlugin] for opening a window and displaying a render target to it.
    ///
    /// When you provide [`DisplayConfig`], it opens a window for you using [`WindowBundle`].
    #[derive(Default, Debug)]
    pub struct RenderToWindow {
        target: Target,
        config: Option<DisplayConfig>,
        dimensions: Option<ScreenDimensions>,
        dirty: bool,
        clear: Option<ClearColor>,
    }

    impl RenderToWindow {
        /// Create RenderToWindow plugin with [`WindowBundle`] using specified config path.
        pub fn from_config_path(path: impl AsRef<Path>) -> Self {
            Self::from_config(DisplayConfig::load(path))
        }

        /// Create RenderToWindow plugin with [`WindowBundle`] using specified config.
        pub fn from_config(display_config: DisplayConfig) -> Self {
            Self {
                config: Some(display_config),
                ..Default::default()
            }
        }

        /// Select render target which will be presented to window.
        pub fn with_target(mut self, target: Target) -> Self {
            self.target = target;
            self
        }

        /// Clear window with specified color every frame.
        pub fn with_clear(mut self, clear: impl Into<ClearColor>) -> Self {
            self.clear = Some(clear.into());
            self
        }
    }

    impl<B: Backend> RenderPlugin<B> for RenderToWindow {
        fn on_build<'a, 'b>(
            &mut self,
            world: &mut World,
            builder: &mut DispatcherBuilder,
        ) -> Result<(), Error> {
            if let Some(config) = self.config.take() {
                WindowBundle::from_config(config).build(world, builder)?;
            }

            Ok(())
        }

        #[allow(clippy::map_clone)]
        fn should_rebuild(&mut self, world: &World) -> bool {
            let new_dimensions = world.resources.get::<ScreenDimensions>();
            use std::ops::Deref;
            if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
                self.dirty = true;
                self.dimensions = new_dimensions.map(|d| d.deref().clone());
                return false;
            }
            self.dirty
        }

        fn on_plan(
            &mut self,
            plan: &mut RenderPlan<B>,
            factory: &mut Factory<B>,
            world: &World,
        ) -> Result<(), Error> {
            self.dirty = false;

            let window = world.resources.get::<Window>().unwrap();

            let surface = factory.create_surface(&window);
            let dimensions = self.dimensions.as_ref().unwrap();
            let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

            let depth_options = ImageOptions {
                kind: window_kind,
                levels: 1,
                format: Format::D32Sfloat,
                clear: Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
            };

            plan.add_root(Target::Main);
            plan.define_pass(
                self.target,
                TargetPlanOutputs {
                    colors: vec![OutputColor::Surface(
                        surface,
                        self.clear.map(ClearValue::Color),
                    )],
                    depth: Some(depth_options),
                },
            )?;

            Ok(())
        }
    }
}

/// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct RenderFlat2D {
    target: Target,
}

impl RenderFlat2D {
    /// Set target to which 2d sprites will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderFlat2D {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder.add_system_desc(Stage::Render, SpriteVisibilitySortingSystemDesc::default());
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, DrawFlat2DDesc::new().builder())?;
            ctx.add(
                RenderOrder::Transparent,
                DrawFlat2DTransparentDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}

/// A [RenderPlugin] for drawing debug lines.
/// Use with [debug_drawing::DebugLines] resource or [debug_drawing::DebugLinesComponent].
#[derive(Default, Debug)]
pub struct RenderDebugLines {
    target: Target,
}

impl RenderDebugLines {
    /// Set target to which debug lines will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderDebugLines {
    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(
                RenderOrder::BeforeTransparent,
                DrawDebugLinesDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}

/// A `RenderPlugin` for forward rendering of 3d objects using flat shading.
pub type RenderFlat3D = RenderBase3D<crate::legion::pass::FlatPassDef>;
/// A `RenderPlugin` for forward rendering of 3d objects using shaded shading.
pub type RenderShaded3D = RenderBase3D<crate::legion::pass::ShadedPassDef>;
/// A `RenderPlugin` for forward rendering of 3d objects using physically-based shading.
pub type RenderPbr3D = RenderBase3D<crate::legion::pass::PbrPassDef>;

/// A `RenderPlugin` for forward rendering of 3d objects.
/// Generic over 3d pass rendering method.
#[derive(derivative::Derivative)]
#[derivative(Default(bound = ""), Debug(bound = ""))]
pub struct RenderBase3D<D: Base3DPassDef> {
    target: Target,
    skinning: bool,
    marker: std::marker::PhantomData<D>,
}

impl<D: Base3DPassDef> RenderBase3D<D> {
    /// Set target to which 3d meshes will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Enable rendering for skinned meshes.
    ///
    /// NOTE: You must register `VertexSkinningBundle` yourself.
    pub fn with_skinning(mut self) -> Self {
        self.skinning = true;
        self
    }
}

impl<B: Backend, D: Base3DPassDef> RenderPlugin<B> for RenderBase3D<D> {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder.add_system_desc(Stage::Render, VisibilitySortingSystemDesc::default());
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        let skinning = self.skinning;
        plan.extend_target(self.target, move |ctx| {
            ctx.add(
                RenderOrder::Opaque,
                DrawBase3DDesc::<B, D>::new()
                    .with_skinning(skinning)
                    .builder(),
            )?;
            ctx.add(
                RenderOrder::Transparent,
                DrawBase3DTransparentDesc::<B, D>::new()
                    .with_skinning(skinning)
                    .builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}

/// RenderPlugin for rendering skyboxes.
#[derive(Default, Debug)]
pub struct RenderSkybox {
    target: Target,
    colors: Option<(Srgb, Srgb)>,
}

impl RenderSkybox {
    /// Create skybox with specified nadir and zenith colors.
    pub fn with_colors(nadir_color: Srgb, zenith_color: Srgb) -> Self {
        Self {
            target: Default::default(),
            colors: Some((nadir_color, zenith_color)),
        }
    }

    /// Set target to which skybox will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderSkybox {
    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        let colors = self.colors;
        plan.extend_target(self.target, move |ctx| {
            let group = if let Some((nadir, zenith)) = colors {
                DrawSkyboxDesc::with_colors(nadir, zenith).builder()
            } else {
                DrawSkyboxDesc::new().builder()
            };

            ctx.add(RenderOrder::AfterOpaque, group)?;
            Ok(())
        });
        Ok(())
    }
}
