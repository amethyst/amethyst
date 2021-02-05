use amethyst_assets::{
    distill_importer::{typetag, SerdeImportable},
    register_asset_type, AssetProcessorSystem, Handle,
};
use amethyst_core::ecs::CommandBuffer;
use amethyst_rendy::sprite::{SpriteRender, SpriteSheet};
use log::error;
use minterpolate::InterpolationPrimitive;
use serde::{Deserialize, Serialize};

use crate::{Animation, AnimationSampling, BlendMethod, Sampler};

/// Sampler primitive for SpriteRender animations
/// Note that sprites can only ever be animated with `Step`, or a panic will occur.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum SpriteRenderPrimitive {
    /// A spritesheet id
    #[serde(skip)]
    SpriteSheet(Handle<SpriteSheet>),
    /// An index into a spritesheet
    SpriteIndex(usize),
}

use type_uuid::TypeUuid;
use uuid::Uuid;

// 8716643e-4d3a-11eb-bdd2-d7a177713b84
impl TypeUuid for Sampler<SpriteRenderPrimitive> {
    const UUID: type_uuid::Bytes =
        *Uuid::from_u128(179562043138858398183947466578368478084).as_bytes();
}
#[typetag::serde]
impl SerdeImportable for Sampler<SpriteRenderPrimitive> {}
register_asset_type!(Sampler<SpriteRenderPrimitive> => Sampler<SpriteRenderPrimitive>; AssetProcessorSystem<Sampler<SpriteRenderPrimitive>>);

impl InterpolationPrimitive for SpriteRenderPrimitive {
    fn add(&self, _: &Self) -> Self {
        panic!("Cannot add SpriteRenderPrimitive")
    }

    fn sub(&self, _: &Self) -> Self {
        panic!("Cannot sub SpriteRenderPrimitive")
    }

    fn mul(&self, _: f32) -> Self {
        panic!("Cannot mul SpriteRenderPrimitive")
    }

    fn dot(&self, _: &Self) -> f32 {
        panic!("Cannot dot SpriteRenderPrimitive")
    }

    fn magnitude2(&self) -> f32 {
        panic!("Cannot magnitude2 SpriteRenderPrimitive")
    }

    fn magnitude(&self) -> f32 {
        panic!("Cannot magnitude SpriteRenderPrimitive")
    }

    fn normalize(&self) -> Self {
        panic!("Cannot normalize SpriteRenderPrimitive")
    }
}

/// Channels that are animatable on `SpriteRender`
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SpriteRenderChannel {
    /// Selecting a spritesheet dynamically
    SpriteSheet,
    /// Selecting a sprite index dynamically
    SpriteIndex,
}

// 9ef0c4bb-45d2-8d45-b418-5001f89cbb0d
impl TypeUuid for Animation<SpriteRender> {
    const UUID: type_uuid::Bytes =
        *Uuid::from_u128(211268164769622779683751576167574715149).as_bytes();
}
register_asset_type!(Animation<SpriteRender> => Animation<SpriteRender>; AssetProcessorSystem<Animation<SpriteRender>>);

impl AnimationSampling for SpriteRender {
    type Primitive = SpriteRenderPrimitive;
    type Channel = SpriteRenderChannel;

    fn apply_sample<'a>(
        &mut self,
        channel: &Self::Channel,
        data: &Self::Primitive,
        _buffer: &mut CommandBuffer,
    ) {
        use self::{SpriteRenderChannel as Channel, SpriteRenderPrimitive as Primitive};
        match (channel, data) {
            (Channel::SpriteSheet, Primitive::SpriteSheet(handle)) => {
                self.sprite_sheet = handle.clone();
            }
            (Channel::SpriteIndex, Primitive::SpriteIndex(index)) => {
                self.sprite_number = *index;
            }

            // Error cases
            (Channel::SpriteSheet, Primitive::SpriteIndex(_)) => {
                let message = "The `SpriteSheet` render channel must be used with \
                               `SpriteRenderPrimitive::SpriteSheet`"
                    .to_string();
                error!("{}", message);
                panic!("{}", message);
            }
            (Channel::SpriteIndex, Primitive::SpriteSheet(_)) => {
                let message = "The `SpriteIndex` render channel must be used with \
                               `SpriteRenderPrimitive::SpriteIndex`"
                    .to_string();
                error!("{}", message);
                panic!("{}", message);
            }
        }
    }

    fn current_sample<'a>(&self, channel: &Self::Channel) -> Self::Primitive {
        use self::{SpriteRenderChannel as Channel, SpriteRenderPrimitive as Primitive};

        match channel {
            Channel::SpriteSheet => Primitive::SpriteSheet(self.sprite_sheet.clone()),
            Channel::SpriteIndex => Primitive::SpriteIndex(self.sprite_number),
        }
    }

    fn default_primitive(_: &Self::Channel) -> Self::Primitive {
        panic!("Blending is not applicable to SpriteRender animation")
    }

    fn blend_method(&self, _: &Self::Channel) -> Option<BlendMethod> {
        None
    }
}
