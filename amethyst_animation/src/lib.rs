extern crate amethyst_assets;
extern crate amethyst_core;
extern crate fnv;
extern crate minterpolate;
#[macro_use]
extern crate serde;
extern crate specs;

pub use self::bundle::{AnimationBundle, SamplingBundle};
pub use self::interpolation::{Interpolate, InterpolationFunction, InterpolationType};
pub use self::resources::{Animation, AnimationCommand, AnimationControl, AnimationHierarchy,
                          AnimationOutput, AnimationSet, ControlState, EndControl, RestState,
                          Sampler, SamplerControl, SamplerControlSet};
pub use self::systems::{AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem,
                        SamplerProcessor};
pub use self::util::*;

mod resources;
mod systems;
mod interpolation;
mod bundle;
mod util;
