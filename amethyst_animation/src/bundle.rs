use std::{hash::Hash, marker};

use crate::{
    resources::AnimationSampling,
    skinning::VertexSkinningSystem,
    systems::{
        AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem, SamplerProcessor,
    },
};

use amethyst_core::{specs::prelude::Component, Result, SimpleDispatcherBuilder, SystemBundle};

/// Bundle for vertex skinning
///
/// This registers `VertexSkinningSystem`.
/// Note that the user must make sure this system runs after `TransformSystem`
#[derive(Default)]
pub struct VertexSkinningBundle<'a: 'b, 'b> {
    dep: &'b [&'a str],
}

impl<'a, 'b> VertexSkinningBundle<'a, 'b> {
    /// Create a new sampling bundle
    pub fn new() -> Self {
        Default::default()
    }

    /// Set dependencies for the `VertexSkinningSystem`
    pub fn with_dep(mut self, dep: &'b [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, 'd, D> SystemBundle<'a, 'b, 'c, D> for VertexSkinningBundle<'c, 'd>
where
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    fn build(self, builder: &mut D) -> Result<()> {
        builder.add(
            VertexSkinningSystem::new(),
            "vertex_skinning_system",
            self.dep,
        );
        Ok(())
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
pub struct SamplingBundle<'a: 'b, 'b, T> {
    name: &'a str,
    dep: &'b [&'a str],
    m: marker::PhantomData<T>,
}

impl<'a, 'b, T> SamplingBundle<'a, 'b, T> {
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
    pub fn with_dep(mut self, dep: &'b [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, 'd, T, D> SystemBundle<'a, 'b, 'c, D> for SamplingBundle<'c, 'd, T>
where
    T: AnimationSampling + Component,
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    fn build(self, builder: &mut D) -> Result<()> {
        builder.add(SamplerProcessor::<T::Primitive>::new(), "", &[]);
        builder.add(SamplerInterpolationSystem::<T>::new(), self.name, self.dep);
        Ok(())
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
pub struct AnimationBundle<'a: 'b, 'b, I, T> {
    animation_name: &'a str,
    sampling_name: &'a str,
    dep: &'b [&'a str],
    m: marker::PhantomData<(I, T)>,
}

impl<'a, 'b, I, T> AnimationBundle<'a, 'b, I, T> {
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
    pub fn with_dep(mut self, dep: &'b [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, 'd, I, T, D> SystemBundle<'a, 'b, 'c, D> for AnimationBundle<'c, 'd, I, T>
where
    I: PartialEq + Eq + Hash + Copy + Send + Sync + 'static,
    T: AnimationSampling + Component + Clone,
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    fn build(self, builder: &mut D) -> Result<()> {
        builder.add(AnimationProcessor::<T>::new(), "", &[]);
        builder.add(
            AnimationControlSystem::<I, T>::new(),
            self.animation_name,
            self.dep,
        );
        SamplingBundle::<T>::new(self.sampling_name)
            .with_dep(&[self.animation_name])
            .build(builder)
    }
}
