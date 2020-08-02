//! Set of predefined implementations of `RenderPlugin` for use with `RenderingBundle`.

use crate::{
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    pass::*,
    sprite_visibility::SpriteVisibilitySortingSystem,
    visibility::VisibilitySortingSystem,
    Backend, Factory,
};
use amethyst_core::ecs::{DispatcherBuilder, World};
use amethyst_error::Error;
use palette::Srgb;
use rendy::graph::render::RenderGroupDesc;

#[cfg(feature = "window")]
pub use window::RenderToWindow;

#[cfg(feature = "window")]
mod window {
    use super::*;
    use crate::{
        bundle::{ImageOptions, OutputColor},
        Format, Kind,
    };
    use amethyst_config::{Config, ConfigError};
    use amethyst_core::{
        ecs::{ReadExpect, SystemData},
        SystemBundle,
    };
    use amethyst_window::{DisplayConfig, ScreenDimensions, Window, WindowBundle};
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
        pub fn from_config_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
            Ok(Self::from_config(DisplayConfig::load(path)?))
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
        /// This function takes linear RGBA. You can convert rgba to linear rgba like so:
        ///
        /// ```
        /// use amethyst_rendy::palette::Srgba;
        /// use amethyst_rendy::RenderToWindow;
        /// use amethyst_window::DisplayConfig;
        ///
        /// let your_red: f32 = 255.;
        /// let your_green: f32 = 160.;
        /// let your_blue: f32 = 122.;
        /// let your_alpha: f32 = 1.0;
        ///
        /// let (r, g, b, a) = Srgba::new(your_red / 255., your_green / 255., your_blue / 255., your_alpha)
        ///     .into_linear()
        ///     .into_components();
        ///
        /// RenderToWindow::from_config(DisplayConfig::default()).with_clear([r, g, b, a]);
        /// ```
        pub fn with_clear(mut self, clear: impl Into<ClearColor>) -> Self {
            self.clear = Some(clear.into());
            self
        }
    }

    impl<B: Backend> RenderPlugin<B> for RenderToWindow {
        fn on_build<'a, 'b>(
            &mut self,
            world: &mut World,
            builder: &mut DispatcherBuilder<'a, 'b>,
        ) -> Result<(), Error> {
            if let Some(config) = self.config.take() {
                WindowBundle::from_config(config).build(world, builder)?;
            }

            Ok(())
        }

        #[allow(clippy::map_clone)]
        fn should_rebuild(&mut self, world: &World) -> bool {
            let new_dimensions = world.try_fetch::<ScreenDimensions>();
            if self.dimensions.as_ref() != new_dimensions.as_deref() {
                self.dirty = true;
                self.dimensions = new_dimensions.map(|d| (*d).clone());
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

            let window = <ReadExpect<'_, Window>>::fetch(world);
            let surface = factory.create_surface(&window);
            let dimensions = self.dimensions.as_ref().unwrap();
            let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

            let depth_options = ImageOptions {
                kind: window_kind,
                levels: 1,
                format: Format::D32Sfloat,
                clear: Some(ClearValue::DepthStencil(ClearDepthStencil(0.0, 0))),
            };

            plan.add_root(Target::Main);
            plan.define_pass(
                self.target,
                crate::bundle::TargetPlanOutputs {
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

/// A `RenderPlugin` for forward rendering of 3d objects using flat shading.
pub type RenderFlat3D = RenderBase3D<crate::pass::FlatPassDef>;
/// A `RenderPlugin` for forward rendering of 3d objects using shaded shading.
pub type RenderShaded3D = RenderBase3D<crate::pass::ShadedPassDef>;
/// A `RenderPlugin` for forward rendering of 3d objects using physically-based shading.
pub type RenderPbr3D = RenderBase3D<crate::pass::PbrPassDef>;

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
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(VisibilitySortingSystem::new(), "visibility_system", &[]);
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
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &[],
        );
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
