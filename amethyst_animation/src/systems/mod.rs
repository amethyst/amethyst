pub use self::control::AnimationControlSystem;
pub use self::sampling::SamplerInterpolationSystem;

use amethyst_assets::Processor;

use resources::{Animation, Sampler};

mod sampling;
mod control;

/// Asset storage processor for `Sampler`
pub type SamplerProcessor = Processor<Sampler>;

/// Asset storage processor for `Animation`
pub type AnimationProcessor = Processor<Animation>;
