use amethyst_core::Transform;
use amethyst_core::cgmath::{InnerSpace, Quaternion, Vector3};

use resources::{AnimationSampling, ApplyData, BlendMethod};
use util::SamplerPrimitive;

/// Channels that can be animated on `Transform`
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransformChannel {
    Translation,
    Rotation,
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
        use util::SamplerPrimitive::*;
        match (channel, *data) {
            (&Translation, Vec3(ref d)) => self.set_position(*d),
            (&Rotation, Vec4(ref d)) => self.set_rotation(*d),
            (&Scale, Scalar(ref d)) => self.set_scale(*d),
            _ => panic!("Attempt to apply invalid sample to Transform"),
        };
    }

    fn current_sample(&self, channel: &Self::Channel, _: &()) -> SamplerPrimitive<f32> {
        use self::TransformChannel::*;
        match channel {
            &Translation => SamplerPrimitive::Vec3(self.position().into()),
            &Rotation => SamplerPrimitive::Vec4(self.rotation().into()),
            &Scale => SamplerPrimitive::Scalar(self.scale().into()),
        }
    }
    fn default_primitive(channel: &Self::Channel) -> Self::Primitive {
        use self::TransformChannel::*;
        match channel {
            &Translation => SamplerPrimitive::Vec3([0.; 3]),
            &Rotation => SamplerPrimitive::Vec4([0.; 4]),
            &Scale => SamplerPrimitive::Scalar(0.),
        }
    }

    fn blend_method(&self, _: &Self::Channel) -> Option<BlendMethod> {
        Some(BlendMethod::Linear)
    }
}
