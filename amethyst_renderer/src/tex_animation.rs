use std::time::Duration;

use amethyst_assets::{Asset, AssetStorage, Handle};
use fnv::FnvHashMap as HashMap;
use specs::VecStorage;

/// An asset handle to sprite sheet metadata.
pub type SpriteSheetDataHandle = Handle<SpriteSheetData>;

/// Meta data for a sprite sheet texture.  Does not contain a texture, only a description of how
/// a texture can be animated.
#[derive(Clone, Debug)]
pub struct SpriteSheetData {
    /// A list of frames in this spritesheet.
    pub frames: Vec<Frame>,
    /// A collection of animations, the first layer contains a list of "animations" and the second
    /// layer contains a list of frame indices within the animation.
    pub animations: Vec<Vec<usize>>,
    /// A mapping between string names and indexes into the first layer of the animations member.
    /// This should only be used when switching animation by string name.
    pub animation_mapping: HashMap<String, usize>,
}

impl Asset for SpriteSheetData {
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl Into<Result<SpriteSheetData, ()>> for SpriteSheetData {
    fn into(self) -> Result<SpriteSheetData, ()> {
        Ok(self)
    }
}

/// A component describing the current state of animation for a sprite sheet.
#[derive(Clone, Debug)]
pub struct SpriteSheetAnimation {
    /// The multiplier for playback speed.  0 is paused, 1 is normal speed.
    /// Supports negative values as well.
    pub playback_speed: f32,
    /// The SpriteSheetData we are animating.
    sprite_sheet_data: SpriteSheetDataHandle,
    /// The animation currently playing, this is an index into the first layer of the animations
    /// in sprite_sheet_data.
    current_animation: usize,
    /// The current frame in the current animation.  This is an index into the second layer of
    /// the animations in sprite_sheet_data.
    current_frame: usize,
    /// How long the current frame has been played for.
    frame_timer: Duration,
}

impl SpriteSheetAnimation {
    /// Create a new SpriteSheetAnimation component.
    ///
    /// ## Parameters
    ///
    /// - `playback_speed`: A multiplier that makes an animation go faster or slower.
    /// - `sprite_sheet_data`: The sprite sheet data to be used.
    /// - `sheet_storage`: The storage that `sprite_sheet_data` is from.
    /// - `starting_animation`: The named case-sensitive animation from 'sprite_sheet_data' that
    /// this should play initially.
    ///
    /// ## Returns
    /// If the `starting_animation` wasn't found then this will return `None`.  This could be due
    /// to one of a few things:
    ///
    /// - The `sprite_sheet_data` hadn't loaded yet.  Handles aren't valid in their first frame of
    /// existence.
    /// - An animation with the name from `starting_animation` wasn't found in `sprite_sheet_data`.
    /// - The `sprite_sheet_data` handle wasn't made with the `sheet_storage` AssetStorage.
    pub fn new(
        playback_speed: f32,
        sprite_sheet_data: SpriteSheetDataHandle,
        sheet_storage: &AssetStorage<SpriteSheetData>,
        starting_animation: &str,
    ) -> Option<SpriteSheetAnimation> {
        sheet_storage
            .get(&sprite_sheet_data)
            .and_then(|data| data.animation_mapping.get(starting_animation).map(|i| *i))
            .map(|current_animation| SpriteSheetAnimation {
                playback_speed,
                sprite_sheet_data,
                current_animation,
                current_frame: 0,
                frame_timer: Duration::from_secs(0),
            })
    }

    /// Sets the animation to the named case-sensitive animation from the `SpriteSheetData`.
    ///
    /// ## Returns
    ///
    /// If the name provided can't be found in the `SpriteSheetData` for this component then this
    /// will return `Err` otherwise it will return `Ok`.
    pub fn set_animation(
        &mut self,
        sheet_storage: &AssetStorage<SpriteSheetData>,
        animation: &str,
    ) -> Result<(), ()> {
        let animation = sheet_storage
            .get(&self.sprite_sheet_data)
            .and_then(|data| data.animation_mapping.get(animation).map(|i| *i));
        match animation {
            Some(animation) => {
                self.current_animation = animation;
                self.current_frame = 0;
                self.frame_timer = Duration::from_secs(0);
                Ok(())
            }
            None => Err(()),
        }
    }

    /// Retrieve frame information for the current frame.
    ///
    /// ## Returns
    ///
    /// Most of the time this should return `Some`.  If it doesn't then either the
    /// `SpriteSheetDataHandle` used in initialization or the `sheet_storage` in this function are
    /// invalid.  Because handles take at least one frame to become valid after being loaded the
    /// handle is the most likely point of failure.
    pub fn current_frame(&self, sheet_storage: &AssetStorage<SpriteSheetData>) -> Option<Frame> {
        sheet_storage.get(&self.sprite_sheet_data).map(|data| {
            data.frames[data.animations[self.current_animation][self.current_frame]].clone()
        })
    }
}

/// A description of a frame in a spritesheet.
#[derive(Clone, Debug)]
pub struct Frame {
    /// Normalized x coordinate, 0 is the left side and 1 is the right side
    pub x: f32,
    /// Normalized y coordinate, 0 is the top and 1 is the bottom
    pub y: f32,
    /// Normalized width, 0 is no length and 1 is the full width of the texture
    pub width: f32,
    /// Normalized height, 0 is no height and 1 is the full height of the texture
    pub height: f32,
    /// The duration this frame should be played for.
    pub duration: Duration,
}
