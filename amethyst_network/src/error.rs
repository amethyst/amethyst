//! Module containing error handling logic.

use err_derive::Error;
use std::io;

/// The `amethyst_network` result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Wrapper for all errors who could occur in `amethyst_network`.
#[derive(Debug, Error)]
pub enum Error {
    /// Error that could occur on the UDP-socket
    // NB: NetworkError does not implement std::error::Error and cannot be used as a cause.
    // But in order to get _some_ useful diagnostics out of it we format it in the message.
    #[error(display = "UDP-error occurred: {}", _0)]
    UdpError(laminar::ErrorKind),
    /// Error that could occur whit IO.
    #[error(display = "IO-error occurred")]
    IoError(#[cause] io::Error),
    /// Error that could occur when serializing whit `bincode`
    #[error(display = "Serialization error occurred")]
    SerializeError(#[cause] bincode::Error),
    /// Error that could occur when sending an `ServerSocketEvent` to some channel.
    #[error(display = "Channel send error occurred")]
    ChannelSendError(#[cause] crossbeam_channel::SendError<laminar::Packet>),
    #[error(display = "Some error has occurred")]
    #[doc(hidden)]
    __Nonexhaustive,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<crossbeam_channel::SendError<laminar::Packet>> for Error {
    fn from(e: crossbeam_channel::SendError<laminar::Packet>) -> Error {
        Error::ChannelSendError(e)
    }
}

impl From<laminar::ErrorKind> for Error {
    fn from(e: laminar::ErrorKind) -> Error {
        Error::UdpError(e)
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Error {
        Error::SerializeError(e)
    }
}
