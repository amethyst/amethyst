extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_renderer;
extern crate fnv;
extern crate hibitset;
extern crate minterpolate;
#[macro_use]
extern crate serde;
extern crate specs;

pub use self::bundle::{AnimationBundle, SamplingBundle, VertexSkinningBundle};
pub use self::interpolation::{Interpolate, InterpolationFunction, InterpolationType};
pub use self::resources::{Animation, AnimationCommand, AnimationControl, AnimationHierarchy,
                          AnimationOutput, AnimationSet, ControlState, EndControl, RestState,
                          Sampler, SamplerControl, SamplerControlSet};
pub use self::skinning::{Joint, Skin, VertexSkinningSystem};
pub use self::systems::{AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem,
                        SamplerProcessor};
pub use self::util::{pause_animation, play_animation, toggle_animation};

mod skinning;
mod resources;
mod systems;
mod interpolation;
mod bundle;
mod util;
