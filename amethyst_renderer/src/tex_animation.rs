use amethyst_assets::{Asset, Handle};
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

impl Into<Result<SpriteSheetData>> for SpriteSheetData {
    fn into(data: Self) -> Result<SpriteSheetData> {
        Ok(data)
    }
}

/// A component describing the current state of animation for a sprite sheet.
#[derive(Clone, Debug)]
pub struct SpriteSheetAnimation {
    /// The multiplier for playback speed.  1.0 is normal speed.
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
    frame_timer: f32,
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
    /// The duration this frame should be played for in seconds.
    pub duration: f32,
}
