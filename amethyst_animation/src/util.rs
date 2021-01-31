use amethyst_assets::{
    distill_importer::{typetag, SerdeImportable},
    register_asset_type, AssetProcessorSystem,
};
use amethyst_core::{
    ecs::*,
    math::{convert, RealField, Vector2, Vector3, Vector4},
    simd::scalar::*,
};
use minterpolate::InterpolationPrimitive;
use serde::{Deserialize, Serialize};

use self::SamplerPrimitive::*;
use crate::{
    resources::{AnimationControlSet, AnimationSampling},
    Sampler,
};

/// Get the animation set for an entity. If none exists, one will be added. If entity is invalid,
/// (eg. removed before) None will be returned.
///
/// ### Type parameters:
///
/// - `I`: identifier type for running animations, only one animation can be run at the same time
///        with the same id
/// - `T`: the component type that the animation applies to
pub fn get_animation_set<'a, I, T, W>(
    world: &'a mut W,
    buffer: &mut CommandBuffer,
    entity: Entity,
) -> Option<&'a mut AnimationControlSet<I, T>>
where
    I: Send + Sync + 'static,
    T: AnimationSampling,
    W: EntityStore,
{
    if let Ok(entry) = world.entry_mut(entity) {
        let c = entry.into_component_mut::<AnimationControlSet<I, T>>();
        if c.is_ok() {
            c.ok()
        } else {
            let set = AnimationControlSet::<I, T>::default();
            buffer.add_component(entity, set);
            None
        }
    } else {
        None
    }
}

/// Sampler primitive
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SamplerPrimitive<S>
where
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
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
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
{
    fn from(arr: [S; 2]) -> Self {
        SamplerPrimitive::Vec2(arr)
    }
}

impl<S> From<[S; 3]> for SamplerPrimitive<S>
where
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
{
    fn from(arr: [S; 3]) -> Self {
        SamplerPrimitive::Vec3(arr)
    }
}

impl<S> From<[S; 4]> for SamplerPrimitive<S>
where
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
{
    fn from(arr: [S; 4]) -> Self {
        SamplerPrimitive::Vec4(arr)
    }
}

impl<S> From<Vector2<S>> for SamplerPrimitive<S>
where
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
{
    fn from(arr: Vector2<S>) -> Self {
        SamplerPrimitive::Vec2(arr.into())
    }
}

impl<S> From<Vector3<S>> for SamplerPrimitive<S>
where
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
{
    fn from(arr: Vector3<S>) -> Self {
        SamplerPrimitive::Vec3(arr.into())
    }
}

impl<S> From<Vector4<S>> for SamplerPrimitive<S>
where
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
{
    fn from(arr: Vector4<S>) -> Self {
        SamplerPrimitive::Vec4(arr.into())
    }
}

use type_uuid::TypeUuid;
use uuid::Uuid;

// e05c777b-575e-4f7a-930e-d8ea83bb227b
impl TypeUuid for Sampler<SamplerPrimitive<f32>> {
    const UUID: type_uuid::Bytes =
        *Uuid::from_u128(298227185745652393685123926184903254651).as_bytes();
}
#[typetag::serde]
impl SerdeImportable for Sampler<SamplerPrimitive<f32>> {}
register_asset_type!(Sampler<SamplerPrimitive<f32>> => Sampler<SamplerPrimitive<f32>>; AssetProcessorSystem<Sampler<SamplerPrimitive<f32>>>);

impl<S> InterpolationPrimitive for SamplerPrimitive<S>
where
    S: RealField + SubsetOf<f32> + SupersetOf<f32>,
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
            Vec3(ref s) => {
                Vec3([
                    mul_f32(s[0], scalar),
                    mul_f32(s[1], scalar),
                    mul_f32(s[2], scalar),
                ])
            }
            Vec4(ref s) => {
                Vec4([
                    mul_f32(s[0], scalar),
                    mul_f32(s[1], scalar),
                    mul_f32(s[2], scalar),
                    mul_f32(s[3], scalar),
                ])
            }
        }
    }

    fn dot(&self, other: &Self) -> f32 {
        convert(match (*self, *other) {
            (Scalar(s), Scalar(o)) => (s * o),
            (Vec2(s), Vec2(o)) => (s[0] * o[0] + s[1] * o[1]),
            (Vec3(s), Vec3(o)) => (s[0] * o[0] + s[1] * o[1] + s[2] * o[2]),
            (Vec4(s), Vec4(o)) => (s[0] * o[0] + s[1] * o[1] + s[2] * o[2] + s[3] * o[3]),
            _ => panic!("Interpolation can not be done between primitives of different types"),
        })
    }

    fn magnitude2(&self) -> f32 {
        self.dot(self)
    }

    fn magnitude(&self) -> f32 {
        match *self {
            Scalar(s) => convert(s),
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

fn mul_f32<T: RealField + SubsetOf<f32> + SupersetOf<f32>>(s: T, scalar: f32) -> T {
    convert::<f32, T>(scalar) * s
}
