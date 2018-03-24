use std::hash::Hash;
use std::marker;

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{ECSBundle, GlobalTransform, Result};
use amethyst_core::specs::prelude::{Component, DispatcherBuilder, World};
use amethyst_renderer::JointTransforms;

use material::MaterialTextureSet;
use resources::{Animation, AnimationControlSet, AnimationHierarchy, AnimationSampling,
                AnimationSet, RestState, Sampler, SamplerControlSet};
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
        let mut transforms = world.write::<GlobalTransform>();
        Ok(builder.with(
            VertexSkinningSystem::new(transforms.track_inserted(), transforms.track_modified()),
            "vertex_skinning_system",
            self.dep,
        ))
    }
}

/// Bundle for only the sampler interpolation.
///
/// Will add `SamplerInterpolationSystem<T>` with the given name.
/// Will also add `SamplerProcessor<T::Primitive>`.
///
/// ### Type parameters:
///
/// - `T`: the component type that sampling should be applied to
#[derive(Default)]
pub struct SamplingBundle<'a, T> {
    name: &'a str,
    dep: &'a [&'a str],
    m: marker::PhantomData<T>,
}

impl<'a, T> SamplingBundle<'a, T> {
    /// Create a new sampling bundle
    ///
    /// ### Parameters:
    ///
    /// - `name`: name of the `SamplerInterpolationSystem`
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            dep: &[],
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
    T: AnimationSampling + Component,
{
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world
            .res
            .entry()
            .or_insert_with(AssetStorage::<Sampler<T::Primitive>>::new);
        world.register::<SamplerControlSet<T>>();
        world.add_resource(MaterialTextureSet::default());
        Ok(builder
            .with(SamplerProcessor::<T::Primitive>::new(), "", &[])
            .with(SamplerInterpolationSystem::<T>::new(), self.name, self.dep))
    }
}

/// Bundle for a complete animation setup including sampler interpolation and animation control.
///
/// This will also add `SamplingBundle`, because it is a dependency of this bundle.
///
/// Will add `AnimationControlSystem<T>` with the given name.
/// Will also add `AnimationProcessor<T>`.
///
/// ### Type parameters:
///
/// - `I`: identifier type for running animations, only one animation can be run at the same time
///        with the same id (per entity)
/// - `T`: the component type that sampling should be applied to
#[derive(Default)]
pub struct AnimationBundle<'a, I, T> {
    animation_name: &'a str,
    sampling_name: &'a str,
    dep: &'a [&'a str],
    m: marker::PhantomData<(I, T)>,
}

impl<'a, I, T> AnimationBundle<'a, I, T> {
    /// Create a new animation bundle
    ///
    /// ### Parameters:
    ///
    /// - `animation_name`: name of the `AnimationControlSystem`
    /// - `sampling_name`: name of the `SamplerInterpolationSystem`
    pub fn new(animation_name: &'a str, sampling_name: &'a str) -> Self {
        Self {
            animation_name,
            sampling_name,
            dep: &[],
            m: marker::PhantomData,
        }
    }

    /// Set dependencies for the `AnimationControlSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, I, T> ECSBundle<'a, 'b> for AnimationBundle<'c, I, T>
where
    I: PartialEq + Eq + Hash + Copy + Send + Sync + 'static,
    T: AnimationSampling + Component + Clone,
{
    fn build(
        self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(AssetStorage::<Animation<T>>::new());
        world.register::<AnimationControlSet<I, T>>();
        world.register::<AnimationHierarchy<T>>();
        world.register::<RestState<T>>();
        world.register::<AnimationSet<I, T>>();
        world.register::<Handle<Animation<T>>>();
        builder = builder.with(AnimationProcessor::<T>::new(), "", &[]).with(
            AnimationControlSystem::<I, T>::new(),
            self.animation_name,
            self.dep,
        );
        SamplingBundle::<T>::new(self.sampling_name)
            .with_dep(&[self.animation_name])
            .build(world, builder)
    }
}
