use std::hash::Hash;
use std::marker;
use std::time::Duration;

use amethyst_assets::{Asset, Handle, Result};
use amethyst_core::cgmath::BaseNum;
use fnv::FnvHashMap;
use specs::{Component, DenseVecStorage, Entity, VecStorage};

use interpolation::InterpolationType;

/// Master trait used to define animation sampling on a component
pub trait AnimationSampling: Send + Sync + 'static {
    /// The channel type
    type Channel: Clone + Hash + Eq + Send + Sync + 'static;
    /// Scalar type
    type Scalar: BaseNum + Send + Sync + 'static;

    /// Apply a sample to a channel
    fn apply_sample(&mut self, channel: &Self::Channel, data: &SamplerPrimitive<Self::Scalar>);

    /// Get the current sample for a channel
    fn current_sample(&self, channel: &Self::Channel) -> SamplerPrimitive<Self::Scalar>;
}

/// Sampler primitive
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SamplerPrimitive<S>
where
    S: BaseNum,
{
    Scalar(S),
    Vec2([S; 2]),
    Vec3([S; 3]),
    Vec4([S; 4]),
}

impl<S> From<[S; 2]> for SamplerPrimitive<S>
where
    S: BaseNum,
{
    fn from(arr: [S; 2]) -> Self {
        SamplerPrimitive::Vec2(arr)
    }
}

impl<S> From<[S; 3]> for SamplerPrimitive<S>
where
    S: BaseNum,
{
    fn from(arr: [S; 3]) -> Self {
        SamplerPrimitive::Vec3(arr)
    }
}

impl<S> From<[S; 4]> for SamplerPrimitive<S>
where
    S: BaseNum,
{
    fn from(arr: [S; 4]) -> Self {
        SamplerPrimitive::Vec4(arr)
    }
}

/// Sampler defines a single animation for a single channel on a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sampler<S>
where
    S: BaseNum,
{
    /// Time of key frames
    pub input: Vec<f32>,
    /// Actual output data to interpolate
    pub output: Vec<SamplerPrimitive<S>>,
    /// How should interpolation be done
    pub ty: InterpolationType,
}

impl<S> Asset for Sampler<S>
where
    S: BaseNum + Send + Sync + 'static,
{
    const NAME: &'static str = "animation::Sampler";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl<S> Into<Result<Sampler<S>>> for Sampler<S>
where
    S: BaseNum,
{
    fn into(self) -> Result<Sampler<S>> {
        Ok(self)
    }
}

/// Defines the hierarchy of nodes that a single animation can control.
/// Attach to the root entity that an animation can be defined for.
/// Only required for animations which target more than a single node.
#[derive(Debug, Clone)]
pub struct AnimationHierarchy {
    pub nodes: FnvHashMap<usize, Entity>,
}

impl Component for AnimationHierarchy {
    type Storage = DenseVecStorage<Self>;
}

/// Defines a single animation.
/// Defines relationships between the node index in `AnimationHierarchy` and a `Sampler` handle.
/// If the animation only target a single node index, `AnimationHierarchy` is not required.
#[derive(Clone, Debug)]
pub struct Animation<T>
where
    T: AnimationSampling,
{
    /// node index -> sampler handle
    pub nodes: Vec<(usize, T::Channel, Handle<Sampler<T::Scalar>>)>,
}

impl<T> Asset for Animation<T>
where
    T: AnimationSampling,
{
    const NAME: &'static str = "animation::Animation";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl<T> Into<Result<Animation<T>>> for Animation<T>
where
    T: AnimationSampling,
{
    fn into(self) -> Result<Animation<T>> {
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
            ControlState::Running(_) => true,
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

/// Control handling of animation/sampler end
#[derive(Debug, Clone)]
pub enum EndControl {
    /// Loop the requested number of iterations, None = loop infinitely
    Loop(Option<u32>),
    /// When duration of sampler/animation is reached, go back to rest state
    Normal,
}

/// Control a single active sampler
#[derive(Clone)]
pub struct SamplerControl<T>
where
    T: AnimationSampling,
{
    /// Channel
    pub channel: T::Channel,
    /// Sampler
    pub sampler: Handle<Sampler<T::Scalar>>,
    /// State of sampling
    pub state: ControlState,
    /// What to do when sampler ends
    pub end: EndControl,
    /// What the transform should return to after end
    pub after: SamplerPrimitive<T::Scalar>,
    // Control the rate of animation, default is 1.0
    // pub rate_multiplier: f32, //TODO
}

/// Sampler control set, containing a set of sampler controllers for a single component.
///
/// We only support a single sampler per channel currently, i.e no animation blending. Blending is
/// however possible to build on top of this by dynamically updating the samplers referenced from
/// here.
#[derive(Clone, Default)]
pub struct SamplerControlSet<T>
where
    T: AnimationSampling,
{
    pub samplers: FnvHashMap<T::Channel, SamplerControl<T>>,
}

impl<T> Component for SamplerControlSet<T>
where
    T: AnimationSampling,
{
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

/// Controls the state of a single running animation on a specific component type
#[derive(Clone, Debug)]
pub struct AnimationControl<T>
where
    T: AnimationSampling,
{
    /// Animation handle
    pub animation: Handle<Animation<T>>,
    /// What to do when animation ends
    pub end: EndControl,
    /// State of animation
    pub state: ControlState,
    /// Animation command
    pub command: AnimationCommand,
    m: marker::PhantomData<T>,
}

impl<T> AnimationControl<T>
where
    T: AnimationSampling,
{
    pub fn new(
        animation: Handle<Animation<T>>,
        end: EndControl,
        state: ControlState,
        command: AnimationCommand,
    ) -> Self {
        AnimationControl {
            animation,
            end,
            state,
            command,
            m: marker::PhantomData,
        }
    }
}

impl<T> Component for AnimationControl<T>
where
    T: AnimationSampling,
{
    type Storage = DenseVecStorage<Self>;
}

/// Attaches to an entity that have animations, with links to all animations that can be run on the
/// entity. Is not used directly by the animation systems, provided for convenience.
pub struct AnimationSet<T>
where
    T: AnimationSampling,
{
    pub animations: Vec<Handle<Animation<T>>>,
}

impl<T> Component for AnimationSet<T>
where
    T: AnimationSampling,
{
    type Storage = DenseVecStorage<Self>;
}
