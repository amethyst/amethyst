//! Provides functions used to play audio.

use std::io::Cursor;

use rodio::Decoder;
use rodio::play_once as rplay_once;

use super::DecoderError;
use super::output::Output;
use super::source::Source;

/// Play a sound once.
///
/// This will return an Error if the loaded audio file in source could not be decoded.
pub fn try_play_once(source: &Source, endpoint: &Output) -> Result<(), DecoderError> {
    match rplay_once(&endpoint.endpoint, Cursor::new(source.clone())) {
        Ok(sink) => {
            sink.detach();
            Ok(())
        }

        /// There is one and only one error that can be returned, which is unrecognized format
        /// See documentation for DecoderError here:
        /// https://docs.rs/rodio/0.5.1/rodio/decoder/enum.DecoderError.html
        Err(err) => {
            eprintln!("Error while playing sound: {:?}", err);
            Err(DecoderError)
        }
    }
}

/// Play a sound once.
///
/// This may silently fail, in order to get error information use `try_play_once`.
pub fn play_once(source: &Source, endpoint: &Output) {
    // This absurd construct is here to keep warnings about an unused result from appearing.
    if let Ok(()) = try_play_once(source, endpoint) {}
}

/// Play a sound n times.
///
/// This may silently fail, in order to get error information use `try_play_n_times`.
pub fn play_n_times(source: &Source, endpoint: &Output, n: u16) {
    // This absurd construct is here to keep warnings about an unused result from appearing.
    if let Ok(()) = try_play_n_times(source, endpoint, n) {}
}

/// Play a sound n times.
///
/// This will return an Error if the loaded audio file in source could not be decoded.
pub fn try_play_n_times(source: &Source, endpoint: &Output, n: u16) -> Result<(), DecoderError> {
    if n > 0 {
        match rplay_once(&endpoint.endpoint, Cursor::new(source.clone())) {
            Ok(sink) => {
                for _ in 1..n {
                    sink.append(Decoder::new(Cursor::new(source.clone())).map_err(
                        |_| DecoderError,
                    )?);
                }
                sink.detach();
                return Ok(());
            }

            // There is one and only one error that can be returned, which is unrecognized format
            // See documentation for DecoderError here:
            // https://docs.rs/rodio/0.5.1/rodio/decoder/enum.DecoderError.html
            Err(err) => {
                eprintln!("Error while playing sound: {:?}", err);
                return Err(DecoderError);
            }
        }
    }
    Ok(())
}
