//! Different types of rendering passes.

use {Encoder, Factory, Result, Target};
use fnv::FnvHashMap as HashMap;
use std::fmt::Debug;

// pub use self::blit::BlitBuffer;
pub use self::clear::ClearTarget;

// mod blit;
mod clear;

/// Passed into the `init()` method of each pass.
pub struct Args<'a>(pub &'a mut Factory, pub &'a HashMap<String, Target>);

/// A discrete rendering pass.
pub trait Pass: Debug + Send + Sync {
    /// Initializes the pass when first added to a pipeline.
    fn init(&mut self, _args: &Args) -> Result<()> { Ok(()) }
    /// Applies the pass to the given target.
    fn apply(&self, enc: &mut Encoder, target: &Target, f64);
}
