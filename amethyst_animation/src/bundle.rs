use amethyst_assets::AssetStorage;
use amethyst_core::{ECSBundle, Result};
use amethyst_renderer::JointTransforms;
use specs::{DispatcherBuilder, World};

use resources::{Animation, AnimationControl, AnimationHierarchy, AnimationSet, Sampler,
                SamplerControlSet};
use skinning::{Joint, Skin, VertexSkinningSystem};
use systems::{AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem,
              SamplerProcessor};

/// Bundle for vertex skinning
///
/// This registers `VertexSkinningSystem`.
/// Note that the user must make sure this system runs after `TransformSystem`
#[derive(Default)]
pub struct VertexSkinningBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> VertexSkinningBundle<'a> {
    /// Create a new sampling bundle
    pub fn new() -> Self {
        Default::default()
    }

    /// Set dependencies for the `VertexSkinningSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> ECSBundle<'a, 'b> for VertexSkinningBundle<'c> {
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.register::<Joint>();
        world.register::<Skin>();
        world.register::<JointTransforms>();
        Ok(builder.add(
            VertexSkinningSystem::new(),
            "vertex_skinning_system",
            self.dep,
        ))
    }
}

/// Bundle for only the sampler interpolation.
///
/// Will add `SamplerInterpolationSystem` with name `sampler_interpolation_system`.
#[derive(Default)]
pub struct SamplingBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> SamplingBundle<'a> {
    /// Create a new sampling bundle
    pub fn new() -> Self {
        Default::default()
    }

    /// Set dependencies for the `SamplerInterpolationSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> ECSBundle<'a, 'b> for SamplingBundle<'c> {
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(AssetStorage::<Sampler>::new());
        world.register::<SamplerControlSet>();

        Ok(builder
            .add(SamplerProcessor::new(), "sampler_processor", &[])
            .add(
                SamplerInterpolationSystem::new(),
                "sampler_interpolation_system",
                self.dep,
            ))
    }
}

/// Bundle for a complete animation setup including sampler interpolation and animation control.
///
/// This will also add `SamplingBundle`, because it is a dependency of this bundle.
///
/// Will add `AnimationControlSystem` with name `animation_control_system`.
#[derive(Default)]
pub struct AnimationBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> AnimationBundle<'a> {
    /// Create a new animation bundle
    pub fn new() -> Self {
        Default::default()
    }

    /// Set dependencies for the `AnimationControlSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> ECSBundle<'a, 'b> for AnimationBundle<'c> {
    fn build(
        self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(AssetStorage::<Animation>::new());
        world.register::<AnimationControl>();
        world.register::<AnimationHierarchy>();
        world.register::<AnimationSet>();
        builder = builder
            .add(AnimationProcessor::new(), "animation_processor", &[])
            .add(
                AnimationControlSystem::new(),
                "animation_control_system",
                self.dep,
            );
        SamplingBundle::new()
            .with_dep(&["animation_control_system"])
            .build(world, builder)
    }
}
