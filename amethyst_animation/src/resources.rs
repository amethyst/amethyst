use std::{cmp::Ordering, fmt::Debug, hash::Hash, marker, time::Duration};

use amethyst_assets::{
    erased_serde::private::serde::{de, de::SeqAccess, ser::SerializeSeq},
    prefab::{
        register_component_type,
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
    Asset, AssetStorage, Handle,
};
use amethyst_core::{
    ecs::*,
    Transform,
};
use derivative::Derivative;
use fnv::FnvHashMap;
use log::debug;
use minterpolate::{get_input_index, InterpolationFunction, InterpolationPrimitive};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use uuid::Uuid;

/// Blend method for sampler blending
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub enum BlendMethod {
    /// Simple linear blending
    Linear,
}

/// Master trait used to define animation sampling on a component
pub trait AnimationSampling: Send + Sync + 'static {
    /// The interpolation primitive
    type Primitive: InterpolationPrimitive + Debug + Clone + Send + Sync + 'static;
    /// An independent grouping or type of functions that operate on attributes of a component
    ///
    /// For example, `translation`, `scaling` and `rotation` are transformation channels independent
    /// of each other, even though they all mutate coordinates of a component.
    type Channel: Debug + Clone + Hash + Eq + Send + Sync + 'static;

    /// Apply a sample to a channel
    fn apply_sample(
        &mut self,
        channel: &Self::Channel,
        data: &Self::Primitive,
        buffer: &mut CommandBuffer,
    );

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
    fn name() -> &'static str {
        "animation::Sampler"
    }
    type Data = Self;
}

/// Define the rest state for a component on an entity
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RestState<T>
where
    T: AnimationSampling + Clone,
{
    state: T,
}

impl<T> RestState<T>
where
    T: AnimationSampling + Clone,
{
    /// Create new rest state
    pub fn new(t: T) -> Self {
        RestState { state: t }
    }

    /// Get the rest state
    pub fn state(&self) -> &T {
        &self.state
    }
}

/// Defines the hierarchy of nodes that a single animation can control.
/// Attached to the root entity that an animation can be defined for.
/// Only required for animations which target more than a single node or entity.
#[derive(Derivative, Debug, Clone, Serialize, Deserialize)]
#[derivative(Default(bound = ""))]
pub struct AnimationHierarchy<T> {
    /// A mapping between indices and entities
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
        Self::default()
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
    pub fn rest_state(&self, world: &SubWorld<'_>, buffer: &mut CommandBuffer)
    where
        T: AnimationSampling + Clone,
    {
        for entity in self.nodes.values() {
            if let Ok(entry) = world.entry_ref(*entity) {
                if entry.get_component::<RestState<T>>().is_err() {
                    if let Some(comp) = entry.get_component::<T>().ok().cloned() {
                        buffer.add_component(*entity, RestState::new(comp));
                    }
                }
            }
        }
    }
}

// d8d687ef-da49-a839-5066-c0d703b99bdc
impl TypeUuid for AnimationSet<usize, Transform> {
    const UUID: type_uuid::Bytes =
        *Uuid::from_u128(288227155745652393685123926184903154652).as_bytes();
}

impl SerdeDiff for AnimationSet<usize, Transform> {
    fn diff<'a, S: SerializeSeq>(
        &self,
        _ctx: &mut DiffContext<'a, S>,
        _other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        _seq: &mut A,
        _ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(AnimationHierarchy<Transform>);

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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T::Channel: Serialize, T::Primitive: Serialize",
    deserialize = "T::Channel: Deserialize<'de>, T::Primitive: Deserialize<'de>",
))]
pub struct Animation<T>
where
    T: AnimationSampling,
{
    /// node index -> sampler handle
    pub nodes: Vec<(usize, T::Channel, Handle<Sampler<T::Primitive>>)>,
}

impl<T> Animation<T>
where
    T: AnimationSampling,
{
    /// Create new empty animation
    pub fn new() -> Self {
        Animation { nodes: vec![] }
    }

    /// Create an animation with a single sampler
    pub fn new_single(
        index: usize,
        channel: T::Channel,
        sampler: Handle<Sampler<T::Primitive>>,
    ) -> Self {
        Animation {
            nodes: vec![(index, channel, sampler)],
        }
    }

    /// Add a sampler to the animation
    pub fn add(
        &mut self,
        node_index: usize,
        channel: T::Channel,
        sampler: Handle<Sampler<T::Primitive>>,
    ) {
        self.nodes.push((node_index, channel, sampler));
    }

    /// Add a sampler to the animation
    pub fn with(
        mut self,
        node_index: usize,
        channel: T::Channel,
        sampler: Handle<Sampler<T::Primitive>>,
    ) -> Self {
        self.nodes.push((node_index, channel, sampler));
        self
    }
}

impl<T> Asset for Animation<T>
where
    T: AnimationSampling,
{
    fn name() -> &'static str {
        "animation::Animation"
    }
    type Data = Self;
}

/// State of animation
#[derive(Debug, Clone, PartialEq)]
pub enum ControlState {
    /// Animation was just requested, not started yet
    Requested,
    /// Deferred start
    Deferred(Duration),
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
        matches!(*self, ControlState::Running(_))
    }

    /// Is the state `Paused`
    pub fn is_paused(&self) -> bool {
        matches!(*self, ControlState::Paused(_))
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
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct SamplerControlSet<T>
where
    T: AnimationSampling,
{
    /// The samplers in this set.
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
        match self
            .samplers
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
        for sampler in self
            .samplers
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
        for sampler in self
            .samplers
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
        let dur = Duration::from_secs_f32(input);
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
            .filter(|t| t.control_id == control_id && t.state != ControlState::Done)
            .map(|c| {
                (
                    samplers
                        .get(&c.sampler)
                        .expect("Referring to a missing sampler"),
                    c,
                )
            })
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

    /// Get the max running duration of the control set
    pub fn get_running_duration(&self, control_id: u64) -> Option<f32> {
        self.samplers
            .iter()
            .filter(|t| t.control_id == control_id)
            .map(|t| {
                if let ControlState::Running(dur) = t.state {
                    dur.as_secs_f32()
                } else {
                    0.
                }
            })
            .max_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Equal))
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
        let dur_s = dur.as_secs_f32();
        let new_index = match (get_input_index(dur_s, &sampler.input), direction) {
            (Some(index), &StepDirection::Forward) if index >= sampler.input.len() - 1 => {
                sampler.input.len() - 1
            }
            (Some(0), &StepDirection::Backward) => 0,
            (Some(index), &StepDirection::Forward) => index + 1,
            (Some(index), &StepDirection::Backward) => index - 1,
            (None, _) => 0,
        };
        control.state = ControlState::Running(Duration::from_secs_f32(sampler.input[new_index]));
    }
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
    /// Only initialize the animation without starting it
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
    /// Id, a value of zero means this has not been initialized yet
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
    /// Creates a new `AnimationControl`
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

/// Defer the start of an animation until the relationship has done this
#[derive(Debug, Clone, PartialEq)]
pub enum DeferStartRelation {
    /// Start animation time duration after relationship started
    Start(f32),
    /// Start animation when relationship ends
    End,
}

#[derive(Debug, Clone)]
pub(crate) struct DeferredStart<I, T>
where
    T: AnimationSampling,
{
    pub animation_id: I,
    pub relation: (I, DeferStartRelation),
    pub control: AnimationControl<T>,
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
    /// The animation set.
    pub animations: Vec<(I, AnimationControl<T>)>,
    pub(crate) deferred_animations: Vec<DeferredStart<I, T>>,
}

impl<I, T> Default for AnimationControlSet<I, T>
where
    T: AnimationSampling,
{
    fn default() -> Self {
        AnimationControlSet {
            animations: Vec::default(),
            deferred_animations: Vec::default(),
        }
    }
}

impl<I, T> AnimationControlSet<I, T>
where
    I: PartialEq + Debug,
    T: AnimationSampling,
{
    /// Is the animation set empty?
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty() && self.deferred_animations.is_empty()
    }

    /// Remove animation from set
    ///
    /// This should be used with care, as this will leave all linked samplers in place. If in
    /// doubt, use `abort()` instead.
    pub fn remove(&mut self, id: I) -> &mut Self {
        if let Some(index) = self.animations.iter().position(|a| a.0 == id) {
            self.animations.remove(index);
        }
        self
    }

    fn set_command(&mut self, id: I, command: AnimationCommand<T>) -> &mut Self {
        if let Some(&mut (_, ref mut control)) = self.animations.iter_mut().find(|a| a.0 == id) {
            control.command = command;
        } else if let Some(ref mut control) = self
            .deferred_animations
            .iter_mut()
            .find(|a| a.animation_id == id)
        {
            control.control.command = command;
        }

        self
    }

    /// Start animation if it exists
    pub fn start(&mut self, id: I) -> &mut Self {
        self.set_command(id, AnimationCommand::Start)
    }

    /// Pause animation if it exists
    pub fn pause(&mut self, id: I) -> &mut Self {
        self.set_command(id, AnimationCommand::Pause)
    }

    /// Toggle animation if it exists
    pub fn toggle(&mut self, id: I) -> &mut Self {
        if let Some(&mut (_, ref mut control)) = self.animations.iter_mut().find(|a| a.0 == id) {
            if control.state.is_running() {
                control.command = AnimationCommand::Pause;
            } else {
                control.command = AnimationCommand::Start;
            }
        }

        self
    }

    /// Set animation rate
    pub fn set_rate(&mut self, id: I, rate_multiplier: f32) -> &mut Self {
        if let Some(&mut (_, ref mut control)) = self.animations.iter_mut().find(|a| a.0 == id) {
            control.rate_multiplier = rate_multiplier;
        }
        if let Some(ref mut control) = self
            .deferred_animations
            .iter_mut()
            .find(|a| a.animation_id == id)
        {
            control.control.rate_multiplier = rate_multiplier;
        }

        self
    }

    /// Step animation
    pub fn step(&mut self, id: I, direction: StepDirection) -> &mut Self {
        self.set_command(id, AnimationCommand::Step(direction))
    }

    /// Set animation input value (point of interpolation)
    pub fn set_input(&mut self, id: I, input: f32) -> &mut Self {
        self.set_command(id, AnimationCommand::SetInputValue(input))
    }

    /// Set blend weights
    pub fn set_blend_weight(&mut self, id: I, weights: Vec<(usize, T::Channel, f32)>) -> &mut Self {
        self.set_command(id, AnimationCommand::SetBlendWeights(weights))
    }

    /// Abort animation
    pub fn abort(&mut self, id: I) -> &mut Self {
        self.set_command(id, AnimationCommand::Abort)
    }

    /// Add animation with the given id, unless it already exists
    pub fn add_animation(
        &mut self,
        id: I,
        animation: &Handle<Animation<T>>,
        end: EndControl,
        rate_multiplier: f32,
        command: AnimationCommand<T>,
    ) -> &mut Self {
        if !self.animations.iter().any(|a| a.0 == id) {
            debug!("Adding animation {:?}", id);
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
        self
    }

    /// Add deferred animation with the given id, unless it already exists
    pub fn add_deferred_animation(
        &mut self,
        id: I,
        animation: &Handle<Animation<T>>,
        end: EndControl,
        rate_multiplier: f32,
        command: AnimationCommand<T>,
        wait_for: I,
        wait_deferred_for: DeferStartRelation,
    ) -> &mut Self {
        if !self.animations.iter().any(|a| a.0 == id) {
            self.deferred_animations.push(DeferredStart {
                animation_id: id,
                relation: (wait_for, wait_deferred_for),
                control: AnimationControl::new(
                    animation.clone(),
                    end,
                    ControlState::Requested,
                    command,
                    rate_multiplier,
                ),
            });
        }
        self
    }

    /// Insert an animation directly
    pub fn insert(&mut self, id: I, control: AnimationControl<T>) -> &mut Self {
        if !self.animations.iter().any(|a| a.0 == id) {
            self.animations.push((id, control));
        }
        self
    }

    /// Check if there is an animation with the given id in the set
    pub fn has_animation(&self, id: I) -> bool {
        self.animations.iter().any(|a| a.0 == id)
    }
}

/// Attaches to an entity that have animations, with links to all animations that can be run on the
/// entity. Is not used directly by the animation systems, provided for convenience.
///
/// ### Type parameters:
///
/// - `T`: the component type that the animation should be applied to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSet<I, T>
where
    I: Eq + Hash,
    T: AnimationSampling,
{
    /// The mapping between `I` and the animation handles.
    pub animations: FnvHashMap<I, Handle<Animation<T>>>,
}

impl<I, T> Default for AnimationSet<I, T>
where
    I: Eq + Hash,
    T: AnimationSampling,
{
    fn default() -> Self {
        AnimationSet {
            animations: FnvHashMap::default(),
        }
    }
}

impl<I, T> AnimationSet<I, T>
where
    I: Eq + Hash,
    T: AnimationSampling,
{
    /// Create
    pub fn new() -> Self {
        AnimationSet {
            animations: FnvHashMap::default(),
        }
    }

    /// Insert an animation in the set
    pub fn insert(&mut self, id: I, handle: Handle<Animation<T>>) -> &mut Self {
        self.animations.insert(id, handle);
        self
    }

    /// Retrieve an animation handle from the set
    pub fn get(&self, id: &I) -> Option<&Handle<Animation<T>>> {
        self.animations.get(id)
    }
}

register_component_type!(AnimationSet<usize, Transform>);

// d8160db6-6dc5-49fd-4c08-8b81739b175c
impl TypeUuid for AnimationHierarchy<Transform> {
    const UUID: type_uuid::Bytes =
        *Uuid::from_u128(287227755745252393685123926184901154652).as_bytes();
}

impl SerdeDiff for AnimationHierarchy<Transform> {
    fn diff<'a, S: SerializeSeq>(
        &self,
        _ctx: &mut DiffContext<'a, S>,
        _other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        _seq: &mut A,
        _ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}
