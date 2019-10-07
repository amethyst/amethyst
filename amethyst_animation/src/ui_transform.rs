use amethyst_core::math::zero;
use amethyst_ui::UiTransform;

use serde::{Deserialize, Serialize};

use crate::{
    resources::{AnimationSampling, ApplyData, BlendMethod},
    util::SamplerPrimitive,
};

/// Channels that can be animated on `UiTransform`
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum UiTransformChannel {
    /// The 2 dimensional position for an UI entity
    Translation,
}

impl<'a> ApplyData<'a> for UiTransform {
    type ApplyData = ();
}

impl AnimationSampling for UiTransform {
    type Primitive = SamplerPrimitive<f32>;
    type Channel = UiTransformChannel;

    fn apply_sample(&mut self, channel: &Self::Channel, data: &SamplerPrimitive<f32>, _: &()) {
        use crate::util::SamplerPrimitive::*;

        use self::UiTransformChannel::*;

        match (channel, *data) {
            (&Translation, Vec2(ref d)) => {
                self.local_x = d[0];
                self.local_y = d[1];
            }
            _ => panic!("Attempt to apply invalid sample to UiTransform"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel, _: &()) -> SamplerPrimitive<f32> {
        use self::UiTransformChannel::*;
        match channel {
            Translation => SamplerPrimitive::Vec2([self.local_x, self.local_y]),
        }
    }
    fn default_primitive(channel: &Self::Channel) -> Self::Primitive {
        use self::UiTransformChannel::*;
        match channel {
            Translation => SamplerPrimitive::Vec2([zero(); 2]),
        }
    }

    fn blend_method(&self, _: &Self::Channel) -> Option<BlendMethod> {
        Some(BlendMethod::Linear)
    }
}
