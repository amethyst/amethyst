use amethyst_core::LocalTransform;
use amethyst_core::cgmath::{InnerSpace, Quaternion, Vector3};

use resources::AnimationSampling;
use util::SamplerPrimitive;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum LocalTransformChannel {
    Translation,
    Rotation,
    Scale,
}

impl AnimationSampling for LocalTransform {
    type Channel = LocalTransformChannel;
    type Primitive = SamplerPrimitive<f32>;

    fn apply_sample(&mut self, channel: &Self::Channel, data: &SamplerPrimitive<f32>) {
        use self::LocalTransformChannel::*;
        use util::SamplerPrimitive::*;
        match (channel, *data) {
            (&Translation, Vec3(ref d)) => self.translation = Vector3::from(*d),
            (&Rotation, Vec4(ref d)) => self.rotation = Quaternion::from(*d).normalize(),
            (&Scale, Vec3(ref d)) => self.scale = Vector3::from(*d),
            _ => panic!("Attempt to apply invalid sample to LocalTransform"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel) -> SamplerPrimitive<f32> {
        use self::LocalTransformChannel::*;
        match channel {
            &Translation => SamplerPrimitive::Vec3(self.translation.into()),
            &Rotation => SamplerPrimitive::Vec4(self.rotation.into()),
            &Scale => SamplerPrimitive::Vec3(self.scale.into()),
        }
    }
}
