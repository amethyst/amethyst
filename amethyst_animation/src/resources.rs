use std::time::Duration;

use amethyst_assets::{Asset, Handle, Result};
use fnv::FnvHashMap;
use specs::{Component, DenseVecStorage, Entity};

use interpolation::InterpolationType;

/// The actual animation data for a single attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationOutput {
    /// Translation is a 3d vector
    Translation(Vec<[f32; 3]>),
    /// Rotation is a quaternion
    Rotation(Vec<[f32; 4]>),
    /// Scale is a 3d vector
    Scale(Vec<[f32; 3]>),
}

/// Sampler defines a single animation for a single attribute of the `LocalTransform` of the entity
/// it is attached to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sampler {
    /// Time of key frames
    pub input: Vec<f32>,
    /// Actual output data to interpolate
    pub output: AnimationOutput,
    /// How should interpolation be done
    pub ty: InterpolationType,
}

impl Asset for Sampler {
    type Data = Self;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl Into<Result<Sampler>> for Sampler {
    fn into(self) -> Result<Sampler> {
        Ok(self)
    }
}

/// Defines the hierarchy of nodes that a single animation can control.
/// Attach to the root entity that an animation can be defined for.
/// Only required for animations which targets more than a single node.
#[derive(Debug, Clone)]
pub struct AnimationHierarchy {
    pub nodes: FnvHashMap<usize, Entity>,
}

impl Component for AnimationHierarchy {
    type Storage = DenseVecStorage<Self>;
}

/// Defines a single animation.
/// Defines relationships between the node index in `AnimationHierarchy` and a `Sampler` handle.
/// If the animation only targets a single node index, `AnimationHierarchy` is not required.
#[derive(Clone, Debug)]
pub struct Animation {
    /// node index -> sampler handle
    pub nodes: Vec<(usize, Handle<Sampler>)>,
}

impl Asset for Animation {
    type Data = Self;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl Into<Result<Animation>> for Animation {
    fn into(self) -> Result<Animation> {
        Ok(self)
    }
}

/// State of animation
#[derive(Debug, Clone, PartialEq)]
pub enum ControlState {
    /// Animation was just requested, not started yet
    Requested,
    /// Animation is running, contains last animation tick, and accumulated duration
    Running(Duration),
    /// Animation is paused at the accumulated duration
    Paused(Duration),
    /// Request termination of the animation
    Abort,
    /// Animation is completed
    Done,
}

impl ControlState {
    /// Is the state `Running`
    pub fn is_running(&self) -> bool {
        match *self {
            ControlState::Running(..) => true,
            _ => false,
        }
    }

    /// Is the state `Paused`
    pub fn is_paused(&self) -> bool {
        match *self {
            ControlState::Paused(_) => true,
            _ => false,
        }
    }
}

/// The rest state for a single attribute
#[derive(Debug, Clone)]
pub enum RestState {
    /// Translation is a 3d vector
    Translation([f32; 3]),
    /// Rotation is a quaternion
    Rotation([f32; 4]),
    /// Scale is a 3d vector
    Scale([f32; 3]),
}

/// Control handling of animation/sampler end
#[derive(Debug, Clone)]
pub enum EndControl {
    /// Loop the requested number of iterations, None = loop infinitely
    Loop(Option<u32>),
    /// When duration of sampler/animation is reached, go back to rest state
    Normal,
}

/// Run the sampler on the attached entity
#[derive(Clone)]
pub struct SamplerControl {
    /// Sampler
    pub sampler: Handle<Sampler>,
    /// State of sampling
    pub state: ControlState,
    /// What to do when sampler ends
    pub end: EndControl,
    /// What the transform should return to after end
    pub after: RestState,
}

/// Sampler control set, containing optional samplers for each of the possible channels.
///
/// Note that the `AnimationOutput` in the `Sampler` referenced, and the `RestState` referenced in
/// each `SamplerControl` should match the attribute channel from the set here. There is no check
/// made by the system that this is the case.
#[derive(Clone, Default)]
pub struct SamplerControlSet {
    pub translation: Option<SamplerControl>,
    pub rotation: Option<SamplerControl>,
    pub scale: Option<SamplerControl>,
}

impl Component for SamplerControlSet {
    type Storage = DenseVecStorage<Self>;
}

/// Animation command
#[derive(Clone, Debug)]
pub enum AnimationCommand {
    /// Start the animation, or unpause if it's paused
    Start,
    /// Pause the animation
    Pause,
    /// Abort the animation, will cause the control object to be removed from the world
    Abort,
}

/// Attaches to an entity, to control what animations are currently active
#[derive(Clone, Debug)]
pub struct AnimationControl {
    /// Animation handle
    pub animation: Handle<Animation>,
    /// What to do when animation ends
    pub end: EndControl,
    /// State of animation
    pub state: ControlState,
    /// Animation command
    pub command: AnimationCommand,
}

impl Component for AnimationControl {
    type Storage = DenseVecStorage<Self>;
}

/// Attaches to an entity that have animations, with links to all animations that can be run on the
/// entity. Is not used directly by the animation systems, provided for convenience.
pub struct AnimationSet {
    pub animations: Vec<Handle<Animation>>,
}

impl Component for AnimationSet {
    type Storage = DenseVecStorage<Self>;
}
