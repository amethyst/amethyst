extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_renderer;
extern crate fnv;
extern crate hibitset;
#[macro_use]
extern crate log;
extern crate minterpolate;
#[macro_use]
extern crate serde;
extern crate shred;
extern crate specs;

pub use self::bundle::{AnimationBundle, SamplingBundle, VertexSkinningBundle};
pub use self::resources::{Animation, AnimationCommand, AnimationControl, AnimationHierarchy,
                          AnimationSampling, AnimationSet, ControlState, EndControl, Sampler,
                          SamplerControl, SamplerControlSet};
pub use self::skinning::{Joint, Skin, VertexSkinningSystem};
pub use self::systems::{AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem,
                        SamplerProcessor};
pub use self::transform::TransformChannel;
pub use self::util::{pause_animation, play_animation, set_animation_rate, toggle_animation,
                     SamplerPrimitive};
pub use minterpolate::{InterpolationFunction, InterpolationPrimitive};

mod skinning;
mod resources;
mod systems;
mod transform;
mod bundle;
mod util;
