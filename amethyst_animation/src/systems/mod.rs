use amethyst_assets::Processor;

pub use self::{
    control::{AnimationControlSystem, AnimationControlSystemDesc},
    sampling::SamplerInterpolationSystem,
};
use crate::resources::{Animation, Sampler};

mod control;
mod sampling;

/// Asset storage processor for `Sampler`
pub type SamplerProcessor<S> = Processor<Sampler<S>>;

/// Asset storage processor for `Animation`
pub type AnimationProcessor<T> = Processor<Animation<T>>;
