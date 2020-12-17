use std::{hash::Hash, marker};

use amethyst_assets::AssetProcessorSystemBundle;
use amethyst_core::ecs::*;
use amethyst_error::Error;
use marker::PhantomData;

use crate::{resources::AnimationSampling, Animation, Sampler};

/// Bundle for vertex skinning
///
/// This registers `VertexSkinningSystem`.
/// Note that the user must make sure this system runs after `TransformSystem`
#[derive(Default, Debug)]
pub struct VertexSkinningBundle;

impl VertexSkinningBundle {
    /// Create a new sampling bundle
    pub fn new() -> Self {
        Default::default()
    }
}

impl SystemBundle for VertexSkinningBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        // FIXME: builder.add_system(VertexSkinningSystem);
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
pub struct SamplingBundle<T> {
    m: marker::PhantomData<T>,
}

impl<'a, T> SystemBundle for SamplingBundle<T>
where
    T: AnimationSampling,
{
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder.add_system(crate::systems::build_sampler_interpolation_system::<T>());
        builder.add_bundle(AssetProcessorSystemBundle::<Sampler<T::Primitive>>::default());

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

impl<'a, I, T> SystemBundle for AnimationBundle<'static, I, T>
where
    I: PartialEq + Eq + Hash + Copy + Send + Sync + 'static,
    T: AnimationSampling + Clone,
{
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder.add_bundle(AssetProcessorSystemBundle::<Animation<T>>::default());
        builder.add_system(crate::systems::build_animation_control_system::<I, T>());
        builder.add_bundle(SamplingBundle::<T> { m: PhantomData });

        Ok(())
    }
}
