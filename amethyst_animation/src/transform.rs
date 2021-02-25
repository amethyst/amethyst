use amethyst_assets::{register_asset_type, AssetProcessorSystem, TypeUuid};
use amethyst_core::{
    ecs::CommandBuffer,
    math::{zero, Quaternion, Unit, Vector3, Vector4},
    transform::Transform,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    resources::{AnimationSampling, BlendMethod},
    util::SamplerPrimitive,
    Animation,
};

/// Channels that can be animated on `Transform`
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransformChannel {
    /// The 3 dimensional cartesian coordinates of an entity
    Translation,
    /// The rotation in 3 dimensional space
    Rotation,
    /// The scale of an entity i.e. how big it is.
    Scale,
}

// feb635f3-752a-456f-96a6-87cff13595b9
impl TypeUuid for Animation<Transform> {
    const UUID: type_uuid::Bytes =
        *Uuid::from_u128(338570003214035303785978659011038647737).as_bytes();
}
register_asset_type!(Animation<Transform> => Animation<Transform>; AssetProcessorSystem<Animation<Transform>>);

impl AnimationSampling for Transform {
    type Primitive = SamplerPrimitive<f32>;
    type Channel = TransformChannel;

    fn apply_sample(
        &mut self,
        channel: &Self::Channel,
        data: &Self::Primitive,
        _buffer: &mut CommandBuffer,
    ) {
        use self::TransformChannel::*;
        use crate::util::SamplerPrimitive::*;

        match (channel, *data) {
            (&Translation, Vec3(ref d)) => {
                self.set_translation_xyz(d[0], d[1], d[2]);
            }
            (&Rotation, Vec4(ref d)) => {
                *self.rotation_mut() = Unit::new_normalize(Quaternion::from(Vector4::from(*d)));
            }
            (&Scale, Vec3(ref d)) => {
                self.set_scale(Vector3::new(d[0], d[1], d[2]));
            }
            _ => panic!("Attempt to apply invalid sample to Transform"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel) -> Self::Primitive {
        use self::TransformChannel::*;
        match channel {
            Translation => SamplerPrimitive::Vec3((*self.translation()).into()),
            Rotation => SamplerPrimitive::Vec4((*self.rotation().as_vector()).into()),
            Scale => SamplerPrimitive::Vec3((*self.scale()).into()),
        }
    }

    fn default_primitive(channel: &Self::Channel) -> Self::Primitive {
        use self::TransformChannel::*;
        match channel {
            Translation => SamplerPrimitive::Vec3([zero(); 3]),
            Rotation => SamplerPrimitive::Vec4([zero(); 4]),
            Scale => SamplerPrimitive::Vec3([zero(); 3]),
        }
    }

    fn blend_method(&self, _: &Self::Channel) -> Option<BlendMethod> {
        Some(BlendMethod::Linear)
    }
}
