use amethyst_core::specs::prelude::Read;
use amethyst_renderer::{SpriteRender, SpriteSheetSet};
use minterpolate::InterpolationPrimitive;

use {AnimationSampling, ApplyData, BlendMethod};

/// Sampler primitive for SpriteRender animations
/// Note that sprites can only ever be animated with `Step`, or a panic will occur.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpriteRenderPrimitive {
    /// A spritesheet id
    SpriteSheet(u64),
    /// An index into a spritesheet
    SpriteIndex(usize),
}

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

impl<'a> ApplyData<'a> for SpriteRender {
    type ApplyData = Read<'a, SpriteSheetSet>;
}

impl AnimationSampling for SpriteRender {
    type Primitive = SpriteRenderPrimitive;
    type Channel = SpriteRenderChannel;

    fn apply_sample(
        &mut self,
        channel: &Self::Channel,
        data: &Self::Primitive,
        sprite_sheet_set: &Read<SpriteSheetSet>,
    ) {
        use self::SpriteRenderChannel as Channel;
        use self::SpriteRenderPrimitive as Primitive;
        match (*channel, *data) {
            (Channel::SpriteSheet, Primitive::SpriteSheet(i)) => {
                if let Some(handle) = sprite_sheet_set.handle(i) {
                    self.sprite_sheet = handle;
                } else {
                    warn!("No sprite sheet handle found for ID: `{}`", i);
                }
            }
            (Channel::SpriteIndex, Primitive::SpriteIndex(index)) => {
                self.sprite_number = index;
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

    fn current_sample(
        &self,
        channel: &Self::Channel,
        sprite_sheet_set: &Read<SpriteSheetSet>,
    ) -> Self::Primitive {
        use self::SpriteRenderChannel as Channel;
        use self::SpriteRenderPrimitive as Primitive;
        match *channel {
            Channel::SpriteSheet => {
                Primitive::SpriteSheet(sprite_sheet_set.id(&self.sprite_sheet).unwrap())
            }
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
