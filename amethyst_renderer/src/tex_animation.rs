use std::fmt::{Display, Error as FmtError, Formatter};
use std::error::Error;

use amethyst_assets::{Asset, AssetStorage, Handle};
use amethyst_core::Time;
use fnv::FnvHashMap as HashMap;
use specs::{Fetch, Join, System, VecStorage, WriteStorage};

use MaterialAnimation;


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

/// Error returned by the `set_animation` function on the `SpriteSheetAnimation`
#[derive(Copy, Clone, Debug)]
pub enum SetAnimationError {
    /// The handle given for the `SpriteSheetData` was invalid.  HINT: Handles are not valid for
    /// their first frame of existence.
    HandleInvalid,
    /// The named animation was not found in the underlying `SpriteSheetData`.
    AnimationNotFound,
}

impl Display for SetAnimationError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str(
            match self {
                &SetAnimationError::AnimationNotFound => "Animation was not found.",
                &SetAnimationError::HandleInvalid => "SpriteSheetData Handle given is invalid."
            }
        )
    }
}

impl Error for SetAnimationError {
    fn description(&self) -> &str {
        match self {
            &SetAnimationError::AnimationNotFound => "Animation was not found.",
            &SetAnimationError::HandleInvalid => "SpriteSheetData Handle given is invalid."
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

/// A duration in seconds expressed via an f32.
pub type FloatDuration = f32;

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
    frame_timer: FloatDuration,
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
    pub fn new(playback_speed: f32, sprite_sheet_data: SpriteSheetDataHandle, sheet_storage: &AssetStorage<SpriteSheetData>, starting_animation: &str) -> Option<SpriteSheetAnimation> {
        sheet_storage.get(&sprite_sheet_data).and_then(|data| data.animation_mapping.get(starting_animation).map(|i| *i)).map(|current_animation| {
            SpriteSheetAnimation {
                playback_speed,
                sprite_sheet_data,
                current_animation,
                current_frame: 0,
                frame_timer: 0.0,
            }
        })

    }

    /// Sets the animation to the named case-sensitive animation from the `SpriteSheetData`.
    ///
    /// ## Returns
    ///
    /// If the name provided can't be found in the `SpriteSheetData` for this component then this
    /// will return `Err` otherwise it will return `Ok`.
    pub fn set_animation(&mut self, sheet_storage: &AssetStorage<SpriteSheetData>, animation: &str) -> Result<(), SetAnimationError> {
        let animation = *sheet_storage.get(&self.sprite_sheet_data).ok_or(SetAnimationError::HandleInvalid)?.animation_mapping.get(animation).ok_or(SetAnimationError::AnimationNotFound)?;
        self.current_animation = animation;
        self.current_frame = 0;
        self.frame_timer = 0.0;
        Ok(())
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
        sheet_storage.get(&self.sprite_sheet_data).map(|data| data.frames[data.animations[self.current_animation][self.current_frame]].clone())
    }

    /// Advance the current animation by the time given.  Normally this will be called by the
    /// engine by default using the regularly running game clock.  This function is made public so
    /// that you can manually advance the animation if you require.
    pub fn advance(&mut self, time: FloatDuration, sheet_storage: &AssetStorage<SpriteSheetData>) -> Result<(), ()> {
        let mut frame_time = self.current_frame(sheet_storage).ok_or(())?.duration;
        self.frame_timer += time * self.playback_speed;
        while self.frame_timer >= frame_time {
            self.current_frame += 1;
            let max_frame = sheet_storage.get(&self.sprite_sheet_data).unwrap().animations[self.current_animation].len() - 1;
            if self.current_frame > max_frame {
                self.current_frame = 0;
            }
            self.frame_timer -= frame_time;
            frame_time = self.current_frame(sheet_storage).unwrap().duration;
        }
        Ok(())
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
    pub duration: FloatDuration,
}

/// A system that automatically advances animations per the system clock.
pub struct TexAnimationSystem;

impl<'a> System<'a> for TexAnimationSystem {
    type SystemData = (
        Fetch<'a, Time>,
        Fetch<'a, AssetStorage<SpriteSheetData>>,
        WriteStorage<'a, MaterialAnimation>
    );

    fn run(&mut self, (time, sheet_storage, mut mat_animation): Self::SystemData) {
        let delta_time = time.delta_seconds();
        for mat_animation in (&mut mat_animation).join() {
            mat_animation.albedo_animation.as_mut().map(|anim| anim.advance(delta_time, &sheet_storage));
            mat_animation.emission_animation.as_mut().map(|anim| anim.advance(delta_time, &sheet_storage));
            mat_animation.normal_animation.as_mut().map(|anim| anim.advance(delta_time, &sheet_storage));
            mat_animation.metallic_animation.as_mut().map(|anim| anim.advance(delta_time, &sheet_storage));
            mat_animation.roughness_animation.as_mut().map(|anim| anim.advance(delta_time, &sheet_storage));
            mat_animation.ambient_occlusion_animation.as_mut().map(|anim| anim.advance(delta_time, &sheet_storage));
            mat_animation.caveat_animation.as_mut().map(|anim| anim.advance(delta_time, &sheet_storage));
        }
    }
}
