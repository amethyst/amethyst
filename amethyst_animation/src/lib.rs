extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_renderer;
extern crate fnv;
extern crate hibitset;
extern crate minterpolate;
#[macro_use]
extern crate serde;
extern crate shred;
extern crate specs;

pub use self::bundle::{AnimationBundle, SamplingBundle, VertexSkinningBundle};
pub use self::interpolation::{Interpolate, InterpolationType};
pub use self::resources::{Animation, AnimationCommand, AnimationControl, AnimationHierarchy,
                          AnimationSampling, AnimationSet, ControlState, EndControl, Sampler,
                          SamplerControl, SamplerControlSet};
pub use self::skinning::{Joint, Skin, VertexSkinningSystem};
pub use self::systems::{AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem,
                        SamplerProcessor};
pub use self::transform::TransformChannel;
pub use self::util::{pause_animation, play_animation, toggle_animation, SamplerPrimitive};

mod skinning;
mod resources;
mod systems;
mod interpolation;
mod transform;
mod bundle;
mod util;
