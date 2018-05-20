//! Loading and playing of audio files.
extern crate amethyst_assets;
extern crate amethyst_core;
extern crate cpal;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
extern crate rodio;
extern crate smallvec;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use self::bundle::AudioBundle;
pub use self::components::*;
pub use self::error::{Error, ErrorKind, Result};
pub use self::formats::{FlacFormat, OggFormat, WavFormat};
pub use self::sink::AudioSink;
pub use self::source::{Source, SourceHandle};
pub use self::systems::*;

pub mod output;

mod bundle;
mod components;
mod end_signal;
mod error;
mod formats;
mod sink;
mod source;
mod systems;
