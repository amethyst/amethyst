//! Module containing error handling logic.

use err_derive::Error;
use std::{io, sync::mpsc};

use crate::server::ServerSocketEvent;

/// The `amethyst_network` result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Wrapper for all errors who could occur in `amethyst_network`.
#[derive(Debug, Error)]
pub enum Error {
    /// Error that could occur on the UDP-socket
    // NB: NetworkError does not implement std::error::Error and cannot be used as a cause.
    // But in order to get _some_ useful diagnostics out of it we format it in the message.
    #[error(display = "UDP-error occurred: {}", _0)]
    UdpError(laminar::error::NetworkError),
    /// Error that could occur whit IO.
    #[error(display = "IO-error occurred")]
    IoError(#[cause] io::Error),
    /// Error that could occur when serializing whit `bincode`
    #[error(display = "Serialization error occurred")]
    SerializeError(#[cause] bincode::Error),
    /// Error that could occur when sending an `ServerSocketEvent` to some channel.
    #[error(display = "Channel send error occurred")]
    ChannelSendError(#[cause] mpsc::SendError<ServerSocketEvent>),
    #[error(display = "Some error has occurred")]
    #[doc(hidden)]
    __Nonexhaustive,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<mpsc::SendError<ServerSocketEvent>> for Error {
    fn from(e: mpsc::SendError<ServerSocketEvent>) -> Error {
        Error::ChannelSendError(e)
    }
}

impl From<laminar::error::NetworkError> for Error {
    fn from(e: laminar::error::NetworkError) -> Error {
        Error::UdpError(e)
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Error {
        Error::SerializeError(e)
    }
}
