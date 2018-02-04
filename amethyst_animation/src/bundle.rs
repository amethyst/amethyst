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
/// Will add `SamplerInterpolationSystem<T>` with the given name.
/// Will also add `SamplerProcessor<T::Primitive>` if it has not been added yet.
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
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        if !world
            .res
            .has_value(ResourceId::new::<AssetStorage<Sampler<T::Primitive>>>())
        {
            world.add_resource(AssetStorage::<Sampler<T::Primitive>>::new());
            builder = builder.add(SamplerProcessor::<T::Primitive>::new(), "", &[]);
        }

        world.register::<SamplerControlSet<T>>();
        Ok(builder.add(SamplerInterpolationSystem::<T>::new(), self.name, self.dep))
    }
}

/// Bundle for a complete animation setup including sampler interpolation and animation control.
///
/// This will also add `SamplingBundle`, because it is a dependency of this bundle.
///
/// Will add `AnimationControlSystem<T>` with the given name.
/// Will also add `AnimationProcessor<T>`
#[derive(Default)]
pub struct AnimationBundle<'a, T> {
    animation_name: &'a str,
    sampling_name: &'a str,
    dep: &'a [&'a str],
    m: marker::PhantomData<T>,
}

impl<'a, T> AnimationBundle<'a, T> {
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

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b> for AnimationBundle<'c, T>
where
    T: AnimationSampling + Component,
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
        builder = builder.add(AnimationProcessor::<T>::new(), "", &[]).add(
            AnimationControlSystem::<T>::new(),
            self.animation_name,
            self.dep,
        );
        SamplingBundle::<T>::new(self.sampling_name)
            .with_dep(&[self.animation_name])
            .build(world, builder)
    }
}
