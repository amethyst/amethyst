use minterpolate::InterpolationPrimitive;
use num_traits::cast::{NumCast, ToPrimitive};
use serde::{Deserialize, Serialize};

use amethyst_core::{
    nalgebra::Real,
    specs::prelude::{Entity, WriteStorage},
};

use crate::resources::{AnimationControlSet, AnimationSampling};

use self::SamplerPrimitive::*;

/// Get the animation set for an entity. If none exists, one will be added. If entity is invalid,
/// (eg. removed before) None will be returned.
///
/// ### Type parameters:
///
/// - `I`: identifier type for running animations, only one animation can be run at the same time
///        with the same id
/// - `T`: the component type that the animation applies to
pub fn get_animation_set<'a, I, T>(
    controls: &'a mut WriteStorage<'_, AnimationControlSet<I, T>>,
    entity: Entity,
) -> Option<&'a mut AnimationControlSet<I, T>>
where
    I: Send + Sync + 'static,
    T: AnimationSampling,
{
    controls
        .entry(entity)
        .ok()
        .map(|entry| entry.or_insert_with(AnimationControlSet::default))
}

/// Sampler primitive
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SamplerPrimitive<S>
where
    S: Real,
{
    /// A single value
    Scalar(S),
    /// Two values
    Vec2([S; 2]),
    /// Three values
    Vec3([S; 3]),
    /// Four values
    Vec4([S; 4]),
}

impl<S> From<[S; 2]> for SamplerPrimitive<S>
where
    S: Real,
{
    fn from(arr: [S; 2]) -> Self {
        SamplerPrimitive::Vec2(arr)
    }
}

impl<S> From<[S; 3]> for SamplerPrimitive<S>
where
    S: Real,
{
    fn from(arr: [S; 3]) -> Self {
        SamplerPrimitive::Vec3(arr)
    }
}

impl<S> From<[S; 4]> for SamplerPrimitive<S>
where
    S: Real,
{
    fn from(arr: [S; 4]) -> Self {
        SamplerPrimitive::Vec4(arr)
    }
}

impl<S> InterpolationPrimitive for SamplerPrimitive<S>
where
    S: Real + ToPrimitive + NumCast,
{
    fn add(&self, other: &Self) -> Self {
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
        match (*self, *other) {
            (Scalar(ref s), Scalar(ref o)) => (*s * *o),
            (Vec2(ref s), Vec2(ref o)) => (s[0] * o[0] + s[1] * o[1]),
            (Vec3(ref s), Vec3(ref o)) => (s[0] * o[0] + s[1] * o[1] + s[2] * o[2]),
            (Vec4(ref s), Vec4(ref o)) => (s[0] * o[0] + s[1] * o[1] + s[2] * o[2] + s[3] * o[3]),
            _ => panic!("Interpolation can not be done between primitives of different types"),
        }
        .to_f32()
        .expect("Unexpected error when converting primitive to f32, possibly under/overflow")
    }

    fn magnitude2(&self) -> f32 {
        self.dot(self)
    }

    fn magnitude(&self) -> f32 {
        match *self {
            Scalar(ref s) => s.to_f32().expect(
                "Unexpected error when converting primitive to f32, possibly under/overflow",
            ),
            Vec2(_) | Vec3(_) | Vec4(_) => self.magnitude2().sqrt(),
        }
    }

    fn normalize(&self) -> Self {
        match *self {
            Scalar(_) => *self,
            Vec2(_) | Vec3(_) | Vec4(_) => self.mul(1. / self.magnitude()),
        }
    }
}

fn mul_f32<T>(s: T, scalar: f32) -> T
where
    T: Real + ToPrimitive + NumCast,
{
    NumCast::from(
        s.to_f32()
            .expect("Unexpected error when converting primitive to f32, possibly under/overflow")
            * scalar,
    )
    .expect("Unexpected error when converting f32 to primitive, possibly under/overflow")
}
