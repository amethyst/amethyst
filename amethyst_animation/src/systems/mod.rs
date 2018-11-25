use amethyst_assets::Processor;

use crate::resources::{Animation, Sampler};

pub use self::{control::AnimationControlSystem, sampling::SamplerInterpolationSystem};

mod control;
mod sampling;

/// Asset storage processor for `Sampler`
pub type SamplerProcessor<S> = Processor<Sampler<S>>;

/// Asset storage processor for `Animation`
pub type AnimationProcessor<T> = Processor<Animation<T>>;
