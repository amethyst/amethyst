use amethyst_assets::{register_asset_type, AssetProcessorSystem, TypeUuid};
use amethyst_core::{ecs::CommandBuffer, math::zero};
use amethyst_ui::UiTransform;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    resources::{AnimationSampling, BlendMethod},
    util::SamplerPrimitive,
    Animation,
};

/// Channels that can be animated on `UiTransform`
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum UiTransformChannel {
    /// The 2 dimensional position for an UI entity
    Translation,
}

impl TypeUuid for Animation<UiTransform> {
    const UUID: type_uuid::Bytes =
        *Uuid::from_u128(338570003214035303785978659011038647737).as_bytes();
}
register_asset_type!(Animation<UiTransform> => Animation<UiTransform>; AssetProcessorSystem<Animation<UiTransform>>);

impl AnimationSampling for UiTransform {
    type Primitive = SamplerPrimitive<f32>;
    type Channel = UiTransformChannel;

    fn apply_sample(
        &mut self,
        channel: &Self::Channel,
        data: &SamplerPrimitive<f32>,
        _buffer: &mut CommandBuffer,
    ) {
        use self::UiTransformChannel::*;
        use crate::util::SamplerPrimitive::*;

        match (channel, *data) {
            (&Translation, Vec2(ref d)) => {
                self.local_x = d[0];
                self.local_y = d[1];
            }
            _ => panic!("Attempt to apply invalid sample to UiTransform"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel) -> SamplerPrimitive<f32> {
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
