//! Provides functions used to play audio.

use std::io::Cursor;

use rodio::{Decoder, play_once as rplay_once, Sink, Source as RSource};

use super::DecoderError;
use super::output::Output;
use super::source::Source;

/// Play a sound once.  A volume of 1.0 is unchanged, while 0.0 is silent.
///
/// This will return an Error if the loaded audio file in source could not be decoded.
pub fn try_play_once(source: &Source, volume: f32, endpoint: &Output) -> Result<(), DecoderError> {
    let mut sink = Sink::new(&endpoint.endpoint);
    match Decoder::new(Cursor::new(source.clone())) {
        Ok(source) => {
            sink.append(source.amplify(volume));
            sink.detach();
            Ok(())
        }

        // There is one and only one error that can be returned, which is unrecognized format
        // See documentation for DecoderError here:
        // https://docs.rs/rodio/0.5.1/rodio/decoder/enum.DecoderError.html
        Err(err) => {
            eprintln!("Error while playing sound: {:?}", err);
            Err(DecoderError)
        }
    }
}

/// Play a sound once.  A volume of 1.0 is unchanged, while 0.0 is silent.
///
/// This may silently fail, in order to get error information use `try_play_once`.
pub fn play_once(source: &Source, volume: f32, endpoint: &Output) {
    // This absurd construct is here to keep warnings about an unused result from appearing.
    let _ = try_play_once(source, volume, endpoint);
}

/// Play a sound n times.  A volume of 1.0 is unchanged, while 0.0 is silent.
///
/// This may silently fail, in order to get error information use `try_play_n_times`.
pub fn play_n_times(source: &Source, volume: f32, endpoint: &Output, n: u16) {
    // This absurd construct is here to keep warnings about an unused result from appearing.
    let _ = try_play_n_times(source, volume, endpoint, n);
}

/// Play a sound n times.  A volume of 1.0 is unchanged, while 0.0 is silent.
///
/// This will return an Error if the loaded audio file in source could not be decoded.
pub fn try_play_n_times(
    source: &Source,
    volume: f32,
    endpoint: &Output,
    n: u16,
) -> Result<(), DecoderError> {
    let mut sink = Sink::new(&endpoint.endpoint);
    for _ in 0..n {
        sink.append(
            Decoder::new(Cursor::new(source.clone()))
                .map_err(|_| DecoderError)?
                .amplify(volume),
        );
    }
    sink.detach();
    Ok(())
}
