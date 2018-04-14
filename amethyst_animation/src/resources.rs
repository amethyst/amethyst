use std::fmt::Debug;
use std::hash::Hash;
use std::marker;
use std::time::Duration;

use amethyst_assets::{Asset, AssetStorage, Handle, Result};
use amethyst_core::specs::{Component, DenseVecStorage, Entity, VecStorage, WriteStorage};
use amethyst_core::timing::{duration_to_secs, secs_to_duration};
use fnv::FnvHashMap;
use minterpolate::{get_input_index, InterpolationFunction, InterpolationPrimitive};

/// Blend method for sampler blending
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub enum BlendMethod {
    Linear,
}

/// Master trait used to define animation sampling on a component
pub trait AnimationSampling: Send + Sync + 'static {
    /// The interpolation primitive
    type Primitive: InterpolationPrimitive + Clone + Copy + Send + Sync + 'static;
    /// An independent grouping or type of functions that operate on attributes of a component
    ///
    /// For example, `translation`, `scaling` and `rotation` are transformation channels independent
    /// of each other, even though they all mutate coordinates of a component.
    type Channel: Debug + Clone + Hash + Eq + Send + Sync + 'static;

    /// Apply a sample to a channel
    fn apply_sample(&mut self, channel: &Self::Channel, data: &Self::Primitive);

    /// Get the current sample for a channel
    fn current_sample(&self, channel: &Self::Channel) -> Self::Primitive;

    /// Get default primitive
    fn default_primitive(channel: &Self::Channel) -> Self::Primitive;

    /// Get blend config
    fn blend_method(&self, channel: &Self::Channel) -> Option<BlendMethod>;
}

/// Sampler defines a single animation for a single channel on a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sampler<T>
where
    T: InterpolationPrimitive,
{
    /// Time of key frames
    ///
    /// A simple example of this for animations that are defined with 4 evenly spaced key frames is
    /// `vec![0., 1., 2., 3.]`.
    pub input: Vec<f32>,
    /// Actual output data to interpolate
    ///
    /// For `input` of size `i`, the `output` size differs depending on the interpolation function.
    /// The following list summarizes the size of the `output` for each interpolation function. For
    /// more details, please click through to each interpolation function's documentation.
    ///
    /// * [Linear][lin]: `i` — `[pos_0, .., pos_n]`
    /// * [Spherical Linear][sph]: `i` — `[pos_0, .., pos_n]`
    /// * [Step][step]: `i` — `[pos_0, .., pos_n]`
    /// * [Catmull Rom Spline][cm]: `i + 2` — `[in_tangent_0, pos_0, .., pos_n, out_tangent_n]`
    /// * [Cubic Spline][cub]: `3 * i` — `[in_tangent_0, pos_0, out_tangent_0, ..]`
    ///
    /// [lin]: https://docs.rs/minterpolate/0.2.2/minterpolate/fn.linear_interpolate.html
    /// [sph]: https://docs.rs/minterpolate/0.2.2/minterpolate/fn.spherical_linear_interpolate.html
    /// [step]: https://docs.rs/minterpolate/0.2.2/minterpolate/fn.step_interpolate.html
    /// [cm]: https://docs.rs/minterpolate/0.2.2/minterpolate/fn.catmull_rom_spline_interpolate.html
    /// [cub]: https://docs.rs/minterpolate/0.2.2/minterpolate/fn.cubic_spline_interpolate.html
    pub output: Vec<T>,
    /// How interpolation should be done
    pub function: InterpolationFunction<T>,
}

impl<T> Asset for Sampler<T>
where
    T: InterpolationPrimitive + Send + Sync + 'static,
{
    const NAME: &'static str = "animation::Sampler";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl<T> Into<Result<Sampler<T>>> for Sampler<T>
where
    T: InterpolationPrimitive,
{
    fn into(self) -> Result<Sampler<T>> {
        Ok(self)
    }
}

/// Define the rest state for a component on an entity
pub struct RestState<T> {
    state: T,
}

impl<T> RestState<T> {
    /// Create new rest state
    pub fn new(t: T) -> Self {
        RestState { state: t }
    }

    /// Get the rest state
    pub fn state(&self) -> &T {
        &self.state
    }
}

impl<T> Component for RestState<T>
where
    T: AnimationSampling,
{
    type Storage = DenseVecStorage<Self>;
}

/// Defines the hierarchy of nodes that a single animation can control.
/// Attached to the root entity that an animation can be defined for.
/// Only required for animations which target more than a single node or entity.
#[derive(Debug, Clone)]
pub struct AnimationHierarchy<T> {
    pub nodes: FnvHashMap<usize, Entity>,
    m: marker::PhantomData<T>,
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::fnv::FnvHashMap::default();
         $( map.insert($key, $val); )*
         map
    }}
}

impl<T> AnimationHierarchy<T>
where
    T: AnimationSampling,
{
    /// Create a new hierarchy
    pub fn new() -> Self {
        AnimationHierarchy {
            nodes: FnvHashMap::default(),
            m: marker::PhantomData,
        }
    }

    /// Create a new hierarchy containing a single given entity
    pub fn new_single(index: usize, entity: Entity) -> Self {
        AnimationHierarchy {
            nodes: hashmap![index => entity],
            m: marker::PhantomData,
        }
    }

    /// Create a new hierarchy with the given entity map
    pub fn new_many(nodes: FnvHashMap<usize, Entity>) -> Self {
        AnimationHierarchy {
            nodes,
            m: marker::PhantomData,
        }
    }

    /// Create rest state for the hierarchy. Will copy the values from the base components for each
    /// entity in the hierarchy.
    pub fn rest_state<F>(&self, get_component: F, states: &mut WriteStorage<RestState<T>>)
    where
        T: AnimationSampling,
        F: Fn(Entity) -> Option<T>,
    {
        for entity in self.nodes.values() {
            if states.get(*entity).is_none() {
                if let Some(comp) = get_component(*entity) {
                    states.insert(*entity, RestState::new(comp));
                }
            }
        }
    }
}

impl<T> Component for AnimationHierarchy<T>
where
    T: AnimationSampling,
{
    type Storage = DenseVecStorage<Self>;
}

/// Defines a single animation.
///
/// An animation is a set of [`Sampler`][sampler]s that should always run together as a unit.
///
/// Defines relationships between the node index in `AnimationHierarchy` and a `Sampler` handle.
/// If the animation only targets a single node index, `AnimationHierarchy` is not required.
///
/// ### Type parameters:
///
/// - `T`: the component type that the animation should be applied to
///
/// [sampler]: struct.Sampler.html
#[derive(Clone, Debug)]
pub struct Animation<T>
where
    T: AnimationSampling,
{
    /// node index -> sampler handle
    pub nodes: Vec<(usize, T::Channel, Handle<Sampler<T::Primitive>>)>,
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
    /// When duration of sampler/animation is reached, do nothing: stay at the last sampled state
    Stay,
}

/// Control a single active sampler
///
/// ### Type parameters:
///
/// - `T`: the component type that the sampling should be applied to
#[derive(Clone)]
pub struct SamplerControl<T>
where
    T: AnimationSampling,
{
    /// Id of the animation control this entry belongs to
    pub control_id: u64,
    /// Channel
    pub channel: T::Channel,
    /// Blend weight
    pub blend_weight: f32,
    /// Sampler
    pub sampler: Handle<Sampler<T::Primitive>>,
    /// State of sampling
    pub state: ControlState,
    /// What to do when sampler ends
    pub end: EndControl,
    /// What the transform should return to after end
    pub after: T::Primitive,
    /// Control the rate of animation, default is 1.0
    pub rate_multiplier: f32,
}

/// Sampler control set, containing a set of sampler controllers for a single component.
///
/// Have support for multiple samplers per channel, will do linear blending between all active
/// samplers. The target component specifies if it can be blended, if it can't, the last added
/// sampler wins.
///
/// ### Type parameters:
///
/// - `T`: the component type that the sampling should be applied to
#[derive(Clone)]
pub struct SamplerControlSet<T>
where
    T: AnimationSampling,
{
    pub samplers: Vec<SamplerControl<T>>,
}

impl<T> Default for SamplerControlSet<T>
where
    T: AnimationSampling,
{
    fn default() -> Self {
        SamplerControlSet {
            samplers: Vec::default(),
        }
    }
}

impl<T> SamplerControlSet<T>
where
    T: AnimationSampling,
{
    /// Set channel control
    pub fn add_control(&mut self, control: SamplerControl<T>) {
        match self.samplers
            .iter()
            .position(|t| t.control_id == control.control_id && t.channel == control.channel)
        {
            Some(index) => {
                self.samplers[index] = control;
            }
            None => {
                self.samplers.push(control);
            }
        }
    }

    /// Clear sampler controls for the given animation
    pub fn clear(&mut self, control_id: u64) {
        self.samplers.retain(|t| t.control_id != control_id);
    }

    /// Check if set is empty
    pub fn is_empty(&self) -> bool {
        self.samplers.is_empty()
    }

    /// Abort control set
    pub fn abort(&mut self, control_id: u64) {
        self.samplers
            .iter_mut()
            .filter(|t| t.control_id == control_id)
            .filter(|t| t.state != ControlState::Done)
            .for_each(|sampler| sampler.state = ControlState::Abort);
    }

    /// Pause control set
    pub fn pause(&mut self, control_id: u64) {
        for sampler in self.samplers
            .iter_mut()
            .filter(|t| t.control_id == control_id)
        {
            sampler.state = match sampler.state {
                ControlState::Running(dur) => ControlState::Paused(dur),
                _ => ControlState::Paused(Duration::from_secs(0)),
            }
        }
    }

    /// Unpause control set
    pub fn unpause(&mut self, control_id: u64) {
        for sampler in self.samplers
            .iter_mut()
            .filter(|t| t.control_id == control_id)
        {
            if let ControlState::Paused(dur) = sampler.state {
                sampler.state = ControlState::Running(dur);
            }
        }
    }

    /// Update rate multiplier
    pub fn set_rate_multiplier(&mut self, control_id: u64, rate_multiplier: f32)
    where
        T: AnimationSampling,
    {
        self.samplers
            .iter_mut()
            .filter(|t| t.control_id == control_id)
            .for_each(|sampler| sampler.rate_multiplier = rate_multiplier);
    }

    /// Forcibly set the input value (point of interpolation)
    pub fn set_input(&mut self, control_id: u64, input: f32)
    where
        T: AnimationSampling,
    {
        let dur = secs_to_duration(input);
        self.samplers
            .iter_mut()
            .filter(|t| t.control_id == control_id)
            .for_each(|sampler| {
                if let ControlState::Running(_) = sampler.state {
                    sampler.state = ControlState::Running(dur);
                }
            });
    }

    /// Check if a control set can be terminated
    pub fn check_termination(&self, control_id: u64) -> bool {
        self.samplers
            .iter()
            .filter(|t| t.control_id == control_id)
            .all(|t| t.state == ControlState::Done || t.state == ControlState::Requested)
    }

    /// Step animation
    pub fn step(
        &mut self,
        control_id: u64,
        samplers: &AssetStorage<Sampler<T::Primitive>>,
        direction: &StepDirection,
    ) {
        self.samplers
            .iter_mut()
            .filter(|t| t.control_id == control_id)
            .filter(|t| t.state != ControlState::Done)
            .map(|c| (samplers.get(&c.sampler).unwrap(), c))
            .for_each(|(s, c)| {
                set_step_state(c, s, direction);
            });
    }

    /// Set blend weight for a sampler
    pub fn set_blend_weight(&mut self, control_id: u64, channel: &T::Channel, blend_weight: f32) {
        self.samplers
            .iter_mut()
            .filter(|t| t.control_id == control_id)
            .filter(|t| t.state != ControlState::Done)
            .filter(|t| t.channel == *channel)
            .for_each(|t| t.blend_weight = blend_weight);
    }
}

fn set_step_state<T>(
    control: &mut SamplerControl<T>,
    sampler: &Sampler<T::Primitive>,
    direction: &StepDirection,
) where
    T: AnimationSampling,
{
    if let ControlState::Running(dur) = control.state {
        let dur_s = duration_to_secs(dur);
        let new_index = match (get_input_index(dur_s, &sampler.input), direction) {
            (Some(index), &StepDirection::Forward) if index >= sampler.input.len() - 1 => {
                sampler.input.len() - 1
            }
            (Some(0), &StepDirection::Backward) => 0,
            (Some(index), &StepDirection::Forward) => index + 1,
            (Some(index), &StepDirection::Backward) => index - 1,
            (None, _) => 0,
        };
        control.state = ControlState::Running(secs_to_duration(sampler.input[new_index]));
    }
}

impl<T> Component for SamplerControlSet<T>
where
    T: AnimationSampling,
{
    type Storage = DenseVecStorage<Self>;
}

/// Used when doing animation stepping (i.e only move forward/backward to discrete input values)
#[derive(Clone, Debug)]
pub enum StepDirection {
    /// Take a step forward
    Forward,
    /// Take a step backward
    Backward,
}

/// Animation command
///
/// ### Type parameters:
///
/// - `T`: the component type that the animation should be applied to
#[derive(Clone, Debug)]
pub enum AnimationCommand<T>
where
    T: AnimationSampling,
{
    /// Start the animation, or unpause if it's paused
    Start,
    /// Step the animation forward/backward (move to the next/previous input value in sequence)
    Step(StepDirection),
    /// Forcibly set current interpolation point for the animation, value in seconds
    SetInputValue(f32),
    /// Set blend weights
    SetBlendWeights(Vec<(usize, T::Channel, f32)>),
    /// Pause the animation
    Pause,
    /// Abort the animation, will cause the control object to be removed from the world
    Abort,
    /// Only initialise the animation without starting it
    Init,
}

/// Controls the state of a single running animation on a specific component type
///
/// ### Type parameters:
///
/// - `T`: the component type that the animation should be applied to
#[derive(Clone, Debug)]
pub struct AnimationControl<T>
where
    T: AnimationSampling,
{
    /// Animation handle
    pub animation: Handle<Animation<T>>,
    /// Id, a value of zero means this has not been initialised yet
    /// (this is done by the control system)
    pub id: u64,
    /// What to do when animation ends
    pub end: EndControl,
    /// State of animation
    pub state: ControlState,
    /// Animation command
    pub command: AnimationCommand<T>,
    /// Control the rate of animation, default is 1.0
    pub rate_multiplier: f32,
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
        command: AnimationCommand<T>,
        rate_multiplier: f32,
    ) -> Self {
        AnimationControl {
            id: 0,
            animation,
            end,
            state,
            command,
            rate_multiplier,
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

/// Contains all currently running animations for an entity.
///
/// Have support for running multiple animations, will do linear blending between all active
/// animations. The target component specifies if it can be blended, if it can't, the last added
/// animation wins.
///
/// ### Type parameters:
///
/// - `I`: identifier type for running animations, only one animation can be run at the same time
///        with the same id
/// - `T`: the component type that the animation should be applied to
#[derive(Clone, Debug)]
pub struct AnimationControlSet<I, T>
where
    T: AnimationSampling,
{
    pub animations: Vec<(I, AnimationControl<T>)>,
}

impl<I, T> Default for AnimationControlSet<I, T>
where
    T: AnimationSampling,
{
    fn default() -> Self {
        AnimationControlSet {
            animations: Vec::default(),
        }
    }
}

impl<I, T> AnimationControlSet<I, T>
where
    I: PartialEq,
    T: AnimationSampling,
{
    /// Is the animation set empty?
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    /// Remove animation from set
    ///
    /// This should be used with care, as this will leave all linked samplers in place. If in
    /// doubt, use `abort()` instead.
    pub fn remove(&mut self, id: I) {
        if let Some(index) = self.animations.iter().position(|a| a.0 == id) {
            self.animations.remove(index);
        }
    }

    fn set_command(&mut self, id: I, command: AnimationCommand<T>) {
        if let Some(&mut (_, ref mut control)) = self.animations.iter_mut().find(|a| a.0 == id) {
            control.command = command;
        }
    }

    /// Start animation if it exists
    pub fn start(&mut self, id: I) {
        self.set_command(id, AnimationCommand::Start);
    }

    /// Pause animation if it exists
    pub fn pause(&mut self, id: I) {
        self.set_command(id, AnimationCommand::Pause);
    }

    /// Toggle animation if it exists
    pub fn toggle(&mut self, id: I) {
        if let Some(&mut (_, ref mut control)) = self.animations.iter_mut().find(|a| a.0 == id) {
            if control.state.is_running() {
                control.command = AnimationCommand::Pause;
            } else {
                control.command = AnimationCommand::Start;
            }
        }
    }

    /// Set animation rate
    pub fn set_rate(&mut self, id: I, rate_multiplier: f32) {
        if let Some(&mut (_, ref mut control)) = self.animations.iter_mut().find(|a| a.0 == id) {
            control.rate_multiplier = rate_multiplier;
        }
    }

    /// Step animation
    pub fn step(&mut self, id: I, direction: StepDirection) {
        self.set_command(id, AnimationCommand::Step(direction));
    }

    /// Set animation input value (point of interpolation)
    pub fn set_input(&mut self, id: I, input: f32) {
        self.set_command(id, AnimationCommand::SetInputValue(input));
    }

    /// Set blend weights
    pub fn set_blend_weight(&mut self, id: I, weights: Vec<(usize, T::Channel, f32)>) {
        self.set_command(id, AnimationCommand::SetBlendWeights(weights));
    }

    /// Abort animation
    pub fn abort(&mut self, id: I) {
        self.set_command(id, AnimationCommand::Abort);
    }

    /// Add animation with the given id, unless it already exists
    pub fn add_animation(
        &mut self,
        id: I,
        animation: &Handle<Animation<T>>,
        end: EndControl,
        rate_multiplier: f32,
        command: AnimationCommand<T>,
    ) {
        if let Some(_) = self.animations.iter().find(|a| a.0 == id) {
            return;
        }
        self.animations.push((
            id,
            AnimationControl::new(
                animation.clone(),
                end,
                ControlState::Requested,
                command,
                rate_multiplier,
            ),
        ));
    }

    /// Check if there is an animation with the given id in the set
    pub fn has_animation(&mut self, id: I) -> bool {
        if let Some(_) = self.animations.iter().find(|a| a.0 == id) {
            true
        } else {
            false
        }
    }
}

impl<I, T> Component for AnimationControlSet<I, T>
where
    I: Send + Sync + 'static,
    T: AnimationSampling,
{
    type Storage = DenseVecStorage<Self>;
}

/// Attaches to an entity that have animations, with links to all animations that can be run on the
/// entity. Is not used directly by the animation systems, provided for convenience.
///
/// ### Type parameters:
///
/// - `T`: the component type that the animation should be applied to
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
