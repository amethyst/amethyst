use crate::{
    resources::AnimationSampling,
    skinning::VertexSkinningSystemDesc,
    systems::{
        build_sampler_processor,
        build_animation_processor
    }, build_sampler_interpolation_system,
};
use amethyst_core::{
    ecs::prelude::*,    
    dispatcher::{DispatcherBuilder, Stage, SystemBundle}
};
use amethyst_error::Error;
use std::{hash::Hash, marker};

/// Bundle for vertex skinning
///
/// This registers `VertexSkinningSystem`.
/// Note that the user must make sure this system runs after `TransformSystem`
#[derive(Default, Debug)]
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

impl<'a, 'c> SystemBundle for VertexSkinningBundle<'c> {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a>,
    ) -> Result<(), Error> {
        todo!("Check this");
        builder.add(
            VertexSkinningSystemDesc::default().build(world),
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
#[derive(Default, Debug)]
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

impl<'a, 'c, T> SystemBundle for SamplingBundle<'c, T>
where
    T: AnimationSampling,
{
    fn build(
        self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'a>,
    ) -> Result<(), Error> {

        builder.add_system(Stage::Begin, build_sampler_interpolation_system::<T>);
        builder.add_system(Stage::Begin, build_sampler_processor::<T::Primitive>());
            
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
#[derive(Default, Debug)]
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

impl<'a, 'b, 'c, I, T> SystemBundle for AnimationBundle<'c, I, T>
where
    I: PartialEq + Eq + Hash + Copy + Send + Sync + 'static,
    T: AnimationSampling + Clone,
{
    fn build(
        self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {

        builder.add_system(Stage::Begin, build_animation_processor::<T>());        

        compile_error!("error what's this?");
        /*builder.add(
            AnimationControlSystemDesc::<I, T>::default().build(world),
            self.animation_name,
            self.dep,
        );*/
        SamplingBundle::<T>::new(self.sampling_name)
            .with_dep(&[self.animation_name])
            .build(world, resources, builder)
    }
}
