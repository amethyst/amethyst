use amethyst_assets::Handle;
use amethyst_core::cgmath::BaseNum;
use amethyst_core::cgmath::num_traits::NumCast;
use minterpolate::InterpolationPrimitive;
use specs::{Entity, WriteStorage};

use resources::{Animation, AnimationCommand, AnimationControl, AnimationSampling, ControlState,
                EndControl, StepDirection};

/// Play a given animation on the given entity.
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation to run
/// - `entity`: entity to run the animation on. Must either have an `AnimationHierarchy` that
///             matches the `Animation`, or only refer to a single node, else the animation will
///             not be run.
/// - `end`: action to perform when the animation has reached its end.
/// - `rate_multiplier`: animation rate to set
pub fn play_animation<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
    end: EndControl,
    rate_multiplier: f32,
) where
    T: AnimationSampling,
{
    match controls.get_mut(entity) {
        Some(ref mut control) if control.animation == *animation => {
            control.command = AnimationCommand::Start
        }
        _ => {}
    }
    if let None = controls.get(entity) {
        controls.insert(
            entity,
            AnimationControl::<T>::new(
                animation.clone(),
                end,
                ControlState::Requested,
                AnimationCommand::Start,
                rate_multiplier,
            ),
        );
    }
}

/// Pause the running animation on the given entity.
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation to run
/// - `entity`: entity the animation is running on. Must either have an `AnimationHierarchy` that
///             matches the `Animation`, or only refer to a single node, else the animation will
///             not be run.
pub fn pause_animation<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
) where
    T: AnimationSampling,
{
    if let Some(ref mut control) = controls.get_mut(entity) {
        if control.animation == *animation && control.state.is_running() {
            control.command = AnimationCommand::Pause;
        }
    }
}

/// Toggle the state between paused and running for the given animation on the given entity.
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation
/// - `entity`: entity to run the animation on. Must either have an `AnimationHierarchy` that
///             matches the `Animation`, or only refer to a single node, else the animation will
///             not be run.
/// - `end`: action to perform when the animation has reached its end.
/// - `rate_multiplier`: animation rate to set
pub fn toggle_animation<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
    end: EndControl,
    rate_multiplier: f32,
) where
    T: AnimationSampling,
{
    if controls
        .get(entity)
        .map(|c| c.state.is_running())
        .unwrap_or(false)
    {
        pause_animation(controls, animation, entity);
    } else {
        play_animation(controls, animation, entity, end, rate_multiplier);
    }
}

/// Set animation rate
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation
/// - `entity`: entity the animation is running on.
/// - `rate_multiplier`: animation rate to set
pub fn set_animation_rate<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
    rate_multiplier: f32,
) where
    T: AnimationSampling,
{
    match controls.get_mut(entity) {
        Some(ref mut control) if control.animation == *animation => {
            control.rate_multiplier = rate_multiplier;
        }
        _ => {}
    }
}

/// Step animation.
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation
/// - `entity`: entity the animation is running on.
/// - `direction`: direction to step the animation
pub fn step_animation<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
    direction: StepDirection,
) where
    T: AnimationSampling,
{
    if let Some(ref mut control) = controls.get_mut(entity) {
        if control.animation == *animation && control.state.is_running() {
            control.command = AnimationCommand::Step(direction);
        }
    }
}

/// Forcibly set animation input value (i.e. the point of interpolation)
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation
/// - `entity`: entity the animation is running on.
/// - `input`: input value to set
pub fn set_animation_input<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
    input: f32,
) where
    T: AnimationSampling,
{
    if let Some(ref mut control) = controls.get_mut(entity) {
        if control.animation == *animation && control.state.is_running() {
            control.command = AnimationCommand::SetInputValue(input);
        }
    }
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

impl<S> InterpolationPrimitive for SamplerPrimitive<S>
where
    S: BaseNum,
{
    fn add(&self, other: &Self) -> Self {
        use self::SamplerPrimitive::*;
        match (*self, *other) {
            (Scalar(ref s), Scalar(ref o)) => Scalar(*s + *o),
            (Vec2(ref s), Vec2(ref o)) => Vec2([s[0] + o[0], s[1] + o[1]]),
            (Vec3(ref s), Vec3(ref o)) => Vec3([s[0] + o[0], s[1] + o[1], s[2] + o[2]]),
            (Vec4(ref s), Vec4(ref o)) => {
                Vec4([s[0] + o[0], s[1] + o[1], s[2] + o[2], s[3] + o[3]])
            }
            _ => panic!("Interpolation can not be done between primitives of different types"),
        }
    }

    fn sub(&self, other: &Self) -> Self {
        use self::SamplerPrimitive::*;
        match (*self, *other) {
            (Scalar(ref s), Scalar(ref o)) => Scalar(*s - *o),
            (Vec2(ref s), Vec2(ref o)) => Vec2([s[0] - o[0], s[1] - o[1]]),
            (Vec3(ref s), Vec3(ref o)) => Vec3([s[0] - o[0], s[1] - o[1], s[2] - o[2]]),
            (Vec4(ref s), Vec4(ref o)) => {
                Vec4([s[0] - o[0], s[1] - o[1], s[2] - o[2], s[3] - o[3]])
            }
            _ => panic!("Interpolation can not be done between primitives of different types"),
        }
    }

    fn mul(&self, scalar: f32) -> Self {
        use self::SamplerPrimitive::*;
        match *self {
            Scalar(ref s) => Scalar(mul_f32(*s, scalar)),
            Vec2(ref s) => Vec2([mul_f32(s[0], scalar), mul_f32(s[1], scalar)]),
            Vec3(ref s) => Vec3([
                mul_f32(s[0], scalar),
                mul_f32(s[1], scalar),
                mul_f32(s[2], scalar),
            ]),
            Vec4(ref s) => Vec4([
                mul_f32(s[0], scalar),
                mul_f32(s[1], scalar),
                mul_f32(s[2], scalar),
                mul_f32(s[3], scalar),
            ]),
        }
    }

    fn dot(&self, other: &Self) -> f32 {
        use self::SamplerPrimitive::*;
        match (*self, *other) {
            (Scalar(ref s), Scalar(ref o)) => (*s * *o).to_f32().unwrap(),
            (Vec2(ref s), Vec2(ref o)) => (s[0] * o[0] + s[1] * o[1]).to_f32().unwrap(),
            (Vec3(ref s), Vec3(ref o)) => {
                (s[0] * o[0] + s[1] * o[1] + s[2] * o[2]).to_f32().unwrap()
            }
            (Vec4(ref s), Vec4(ref o)) => (s[0] * o[0] + s[1] * o[1] + s[2] * o[2] + s[3] * o[3])
                .to_f32()
                .unwrap(),
            _ => panic!("Interpolation can not be done between primitives of different types"),
        }
    }

    fn magnitude2(&self) -> f32 {
        self.dot(self)
    }

    fn magnitude(&self) -> f32 {
        use self::SamplerPrimitive::*;
        match *self {
            Scalar(ref s) => s.to_f32().unwrap(),
            Vec2(_) | Vec3(_) | Vec4(_) => self.magnitude2().sqrt(),
        }
    }

    fn normalize(&self) -> Self {
        use self::SamplerPrimitive::*;
        match *self {
            Scalar(_) => *self,
            Vec2(_) | Vec3(_) | Vec4(_) => self.mul(1. / self.magnitude()),
        }
    }
}

fn mul_f32<T>(s: T, scalar: f32) -> T
where
    T: BaseNum,
{
    NumCast::from(s.to_f32().unwrap() * scalar).unwrap()
}
