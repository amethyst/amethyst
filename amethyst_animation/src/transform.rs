use amethyst_core::Transform;
use amethyst_core::cgmath::{InnerSpace, Quaternion, Vector3};

use resources::AnimationSampling;
use util::SamplerPrimitive;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransformChannel {
    Translation,
    Rotation,
    Scale,
}

impl AnimationSampling for Transform {
    type Channel = TransformChannel;
    type Primitive = SamplerPrimitive<f32>;

    fn apply_sample(&mut self, channel: &Self::Channel, data: &SamplerPrimitive<f32>) {
        use self::TransformChannel::*;
        use util::SamplerPrimitive::*;
        match (channel, *data) {
            (&Translation, Vec3(ref d)) => self.translation = Vector3::from(*d),
            (&Rotation, Vec4(ref d)) => self.rotation = Quaternion::from(*d).normalize(),
            (&Scale, Vec3(ref d)) => self.scale = Vector3::from(*d),
            _ => panic!("Attempt to apply invalid sample to Transform"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel) -> SamplerPrimitive<f32> {
        use self::TransformChannel::*;
        match channel {
            &Translation => SamplerPrimitive::Vec3(self.translation.into()),
            &Rotation => SamplerPrimitive::Vec4(self.rotation.into()),
            &Scale => SamplerPrimitive::Vec3(self.scale.into()),
        }
    }
}
