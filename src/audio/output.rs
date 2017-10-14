//! Provides structures and functions used to get audio outputs.

// We have to use types from this to provide an output iterator type.
extern crate cpal;

use std::fmt::{Debug, Formatter, Result as FmtResult};

use self::cpal::{default_endpoint, endpoints};
use self::cpal::EndpointsIterator;
use rodio::Endpoint;

/// A speaker(s) through which audio can be played.
pub struct Output {
    pub(crate) endpoint: Endpoint,
}

impl Output {
    /// Gets the name of the output
    pub fn name(&self) -> String {
        self.endpoint.name()
    }
}

impl Debug for Output {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("Output { endpoint: ")?;
        formatter.write_str(self.name().as_str())?;
        formatter.write_str(" }")?;
        Ok(())
    }
}

/// An iterator over outputs
pub struct OutputIterator {
    input: EndpointsIterator,
}

impl Iterator for OutputIterator {
    type Item = Output;

    fn next(&mut self) -> Option<Output> {
        self.input.next().map(|re| Output { endpoint: re })
    }
}

/// Get the default output, returns none if no outputs are available.
pub fn default_output() -> Option<Output> {
    default_endpoint().map(|re| Output { endpoint: re })
}

/// Get a list of outputs available to the system.
pub fn outputs() -> OutputIterator {
    OutputIterator { input: endpoints() }
}
