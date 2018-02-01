use std::hash::Hash;
use std::marker;

use amethyst_assets::AssetStorage;
use amethyst_core::{ECSBundle, Result};
use amethyst_renderer::JointTransforms;
use shred::ResourceId;
use specs::{Component, DispatcherBuilder, World};

use resources::{Animation, AnimationControl, AnimationHierarchy, AnimationSampling, AnimationSet,
                Sampler, SamplerControlSet};
use skinning::{Joint, Skin, VertexSkinningSystem};
use systems::{AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem,
              SamplerProcessor};

const DEP: [&str; 0] = [];

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
pub struct SamplingBundle<'a, T> {
    dep: &'a [&'a str],
    m: marker::PhantomData<T>,
}

impl<'a, T> SamplingBundle<'a, T> {
    /// Create a new sampling bundle
    pub fn new() -> Self {
        Self {
            dep: &DEP,
            m: marker::PhantomData,
        }
    }

    /// Set dependencies for the `SamplerInterpolationSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b> for SamplingBundle<'c, T>
where
    T: AnimationSampling + Component + Send + Sync + 'static,
    T::Channel: Hash + Eq + Send + Sync + 'static,
    T::Scalar: Send + Sync + 'static,
{
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        if !world
            .res
            .has_value(ResourceId::new::<AssetStorage<Sampler<T::Scalar>>>())
        {
            world.add_resource(AssetStorage::<Sampler<T::Scalar>>::new());
        }
        world.register::<SamplerControlSet<T>>();

        Ok(builder
            .add(
                SamplerProcessor::<T::Scalar>::new(),
                "sampler_processor",
                &[],
            )
            .add(
                SamplerInterpolationSystem::<T>::new(),
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
pub struct AnimationBundle<'a, T> {
    dep: &'a [&'a str],
    m: marker::PhantomData<T>,
}

impl<'a, T> AnimationBundle<'a, T> {
    /// Create a new animation bundle
    pub fn new() -> Self {
        Self {
            dep: &DEP,
            m: marker::PhantomData,
        }
    }

    /// Set dependencies for the `AnimationControlSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b> for AnimationBundle<'c, T>
where
    T: AnimationSampling + Component + Send + Sync + 'static,
    T::Channel: Clone + Hash + Eq + Send + Sync + 'static,
    T::Scalar: Send + Sync + 'static,
{
    fn build(
        self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(AssetStorage::<Animation<T>>::new());
        world.register::<AnimationControl<T>>();
        world.register::<AnimationHierarchy>();
        world.register::<AnimationSet<T>>();
        builder = builder
            .add(AnimationProcessor::<T>::new(), "animation_processor", &[])
            .add(
                AnimationControlSystem::<T>::new(),
                "animation_control_system",
                self.dep,
            );
        SamplingBundle::<T>::new()
            .with_dep(&["animation_control_system"])
            .build(world, builder)
    }
}
