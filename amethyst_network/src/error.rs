//! Module containing error handling logic.

use std::{
    fmt::{self, Display, Formatter},
    io,
    sync::mpsc,
};

use crate::server::ServerSocketEvent;

/// The `amethyst_network` result type.
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Wrapper for all errors who could occur in `amethyst_network`.
#[derive(Debug)]
pub enum ErrorKind {
    /// Error that could occur on the UDP-socket
    UdpError(laminar::error::NetworkError),
    /// Error that could occur whit IO.
    IoError(io::Error),
    /// Error that could occur when serializing whit `bincode`
    SerializeError(bincode::ErrorKind),
    /// Error that could occur when sending an `ServerSocketEvent` to some channel.
    ChannelSendError(mpsc::SendError<ServerSocketEvent>),
    #[doc(hidden)]
    __Nonexhaustive,
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ErrorKind::IoError(ref e) => Some(e),
            ErrorKind::ChannelSendError(ref e) => Some(e),
            ErrorKind::SerializeError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            ErrorKind::UdpError(_) => write!(fmt, "UDP-error occurred"),
            ErrorKind::IoError(_) => write!(fmt, "IO-error occurred"),
            ErrorKind::ChannelSendError(_) => write!(fmt, "Channel send error occurred"),
            ErrorKind::SerializeError(_) => write!(fmt, "Channel send error occurred"),
            _ => write!(fmt, "Some error has occurred"),
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(e: io::Error) -> ErrorKind {
        ErrorKind::IoError(e)
    }
}

impl From<mpsc::SendError<ServerSocketEvent>> for ErrorKind {
    fn from(e: mpsc::SendError<ServerSocketEvent>) -> ErrorKind {
        ErrorKind::ChannelSendError(e)
    }
}

impl From<laminar::error::NetworkError> for ErrorKind {
    fn from(e: laminar::error::NetworkError) -> ErrorKind {
        ErrorKind::UdpError(e)
    }
}

impl From<Box<bincode::ErrorKind>> for ErrorKind {
    fn from(e: Box<bincode::ErrorKind>) -> ErrorKind {
        ErrorKind::SerializeError(*e)
    }
}
