pub use self::control::AnimationControlSystem;
pub use self::sampling::SamplerInterpolationSystem;

use amethyst_assets::Processor;

use resources::{Animation, Sampler};

mod control;
mod sampling;

/// Asset storage processor for `Sampler`
pub type SamplerProcessor<S> = Processor<Sampler<S>>;

/// Asset storage processor for `Animation`
pub type AnimationProcessor<T> = Processor<Animation<T>>;
