use std::{hash::Hash, marker};

use amethyst_core::ecs::*;
use derivative::Derivative;
use marker::PhantomData;

use crate::{resources::AnimationSampling, skinning::VertexSkinningSystem};

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
    ) -> amethyst_core::Result<()> {
        builder.add_system(VertexSkinningSystem::default());
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
    T: AnimationSampling + std::fmt::Debug,
{
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> amethyst_core::Result<()> {
        builder.add_system(crate::systems::sampling::SamplerInterpolationSystem::<T>::default());

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
#[derive(Derivative, Debug)]
#[derivative(Default)]
pub struct AnimationBundle<I, T> {
    m: marker::PhantomData<(I, T)>,
}

impl<I, T> SystemBundle for AnimationBundle<I, T>
where
    I: std::fmt::Debug + PartialEq + Eq + Hash + Copy + Send + Sync + 'static,
    T: AnimationSampling + Clone + std::fmt::Debug,
{
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> amethyst_core::Result<()> {
        builder.add_bundle(SamplingBundle::<T> { m: PhantomData });
        builder.add_system(crate::systems::control::AnimationControlSystem::<I, T>::default());

        Ok(())
    }
}
