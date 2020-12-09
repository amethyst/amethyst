use amethyst_core::{
    math::{zero, Quaternion, Unit, Vector3, Vector4},
    Transform,
};
use serde::{Deserialize, Serialize};

use crate::{
    resources::{AnimationSampling, ApplyData, BlendMethod},
    util::SamplerPrimitive,
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

impl<'a> ApplyData<'a> for Transform {
    type ApplyData = ();
}

impl AnimationSampling for Transform {
    type Primitive = SamplerPrimitive<f32>;
    type Channel = TransformChannel;

    fn apply_sample(&mut self, channel: &Self::Channel, data: &SamplerPrimitive<f32>, _: &()) {
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

    fn current_sample(&self, channel: &Self::Channel, _: &()) -> SamplerPrimitive<f32> {
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
