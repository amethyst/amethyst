//! Provides computer graphics animation functionality.
//!
//! Animation on a single entity comprises of one or more [`Sampler`][sampler]s. Each sampler
//! operates on a [`Channel`][channel]. Thus, for a single entity, conceptually each
//! `(Channel, Sampler)` pair is enough to define one part the animation, and a
//! `Vec<(Channel, Sampler)>` defines the whole animation.
//!
//! In a more complex situation, an object in game may be made up of multiple entities. Say you have
//! a dragon monster, that is defined by a skinned mesh that has a skeleton with 10 joints. Each
//! joint will then be an `Entity`. Our animation definition holds the samplers to run for the whole
//! object. To animate each of the entities of this complex object, we need a way to link the
//! sampler to the each of the entities.
//!
//! Animation definitions are persistent and can be stored on disk. Entities however, are not. To
//! link the right sampler to the right entity, when we construct each of the entities such as the
//! joints, we track it with an index, called the `node_index`.
//!
//! The following list might help to illustrate the scenario:
//!
//! | node index | entity               |
//! | ---------: | -------------------- |
//! |          0 | body ("main" entity) |
//! |          1 | head                 |
//! |          2 | left left            |
//! |          3 | right left           |
//! |        ... | ...                  |
//!
//! The node index to `Entity` mapping is stored in [`AnimationHierarchy`][ani_hie].
//!
//! Back to the animation definition, we also record the `node_index` in the tuple, which we call a
//! "node". Each node is now `(node_index, Channel, Sampler)` (conceptually &mdash; in code the
//! tuple holds references instead of the complete object). Hence, each node holds the information
//! of what channel the sampler belongs to, and which entity it should be applied to.
//!
//! So what happens for the nodes where we only have one entity? Right now Amethyst requires you to
//! assign it node index `0`.
//!
//! # Examples
//!
//! The [`animation`][ex_ani] and [`gltf`][ex_gltf] examples demonstrate usage of this crate.
//!
//! [sampler]: struct.Sampler.html
//! [channel]: trait.AnimationSampling.html#associatedtype.Channel
//! [ani_hie]: struct.AnimationHierarchy.html
//! [ex_ani]: https://github.com/amethyst/amethyst/tree/master/examples/animation
//! [ex_gltf]: https://github.com/amethyst/amethyst/tree/master/examples/gltf

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub use minterpolate::{InterpolationFunction, InterpolationPrimitive};

pub use self::{
    bundle::{AnimationBundle, SamplingBundle, VertexSkinningBundle},
    material::{MaterialChannel, MaterialPrimitive},
    resources::{
        Animation, AnimationCommand, AnimationControl, AnimationControlSet, AnimationHierarchy,
        AnimationSampling, AnimationSet, BlendMethod, ControlState, DeferStartRelation, EndControl,
        RestState, Sampler, SamplerControl, SamplerControlSet, StepDirection,
    },
    skinning::{Joint, Skin, VertexSkinningSystem},
    sprite::{SpriteRenderChannel, SpriteRenderPrimitive},
    transform::TransformChannel,
    util::{get_animation_set, SamplerPrimitive},
};

mod bundle;
mod material;
mod resources;
mod skinning;
mod sprite;
mod systems;
mod transform;
#[cfg(feature = "ui")]
mod ui_transform;
mod util;
