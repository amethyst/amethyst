extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_renderer;
extern crate fnv;
extern crate hibitset;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate minterpolate;
#[macro_use]
extern crate serde;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use self::bundle::{AnimationBundle, SamplingBundle, VertexSkinningBundle};
pub use self::resources::{Animation, AnimationCommand, AnimationControl, AnimationControlSet,
                          AnimationHierarchy, AnimationSampling, AnimationSet, ApplyData,
                          BlendMethod, ControlState, EndControl, Sampler, SamplerControl,
                          SamplerControlSet, StepDirection};
pub use self::skinning::{Joint, Skin, VertexSkinningSystem};
pub use self::systems::{AnimationControlSystem, AnimationProcessor, SamplerInterpolationSystem,
                        SamplerProcessor};
pub use self::transform::TransformChannel;
pub use self::material::{MaterialPrimitive, MaterialChannel, MaterialTextureSet};
pub use self::util::{get_animation_set, SamplerPrimitive};
pub use minterpolate::{InterpolationFunction, InterpolationPrimitive};

mod skinning;
mod resources;
mod systems;
mod material;
mod transform;
mod bundle;
mod util;
