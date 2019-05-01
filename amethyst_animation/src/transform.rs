use amethyst_core::{
    math::{zero, Quaternion, RealField, Unit, Vector3},
    Transform,
};

use num_traits::NumCast;
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

impl<'a, N: RealField> ApplyData<'a> for Transform<N> {
    type ApplyData = ();
}

impl<N: RealField + NumCast> AnimationSampling for Transform<N> {
    type Primitive = SamplerPrimitive<N>;
    type Channel = TransformChannel;

    fn apply_sample(&mut self, channel: &Self::Channel, data: &SamplerPrimitive<N>, _: &()) {
        use crate::util::SamplerPrimitive::*;

        use self::TransformChannel::*;

        match (channel, *data) {
            (&Translation, Vec3(ref d)) => {
                self.set_translation_xyz(d[0], d[1], d[2]);
            }
            (&Rotation, Vec4(ref d)) => {
                *self.rotation_mut() = Unit::new_normalize(Quaternion::new(d[0], d[1], d[2], d[3]));
            }
            (&Scale, Vec3(ref d)) => {
                self.set_scale(Vector3::new(d[0], d[1], d[2]));
            }
            _ => panic!("Attempt to apply invalid sample to Transform"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel, _: &()) -> SamplerPrimitive<N> {
        use self::TransformChannel::*;
        match channel {
            Translation => SamplerPrimitive::Vec3((*self.translation()).into()),
            Rotation => SamplerPrimitive::Vec4({
                let c = self.rotation().as_ref().coords;
                [c.w, c.x, c.y, c.z]
            }),
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
