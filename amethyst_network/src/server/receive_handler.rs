//! The receive handler is used to communicate between a receiving thread and it's owner.
//! The `ReceiveHandler` communicates via `mpsc::channel`s.

use crate::server::ServerSocketEvent;
use std::{
    sync::mpsc::{Iter, Receiver},
    thread::JoinHandle,
};

/// Handler to access the internals of a receiving socket.
pub struct ReceiveHandler {
    /// handle that should be used for reading received packets from a given socket
    pub receiver: Receiver<ServerSocketEvent>,
    /// thread handle to the thread that receives packets on a given socket
    _thread_handle: JoinHandle<()>,
}

impl ReceiveHandler {
    /// Create a new receive handler by specifying:
    /// 1: the max throughput in packets a host can read at once.
    /// 2: handle that should be used for reading received packets from a given socket.
    /// 3: handle that should be used for reading received packets from a given socket.
    pub fn new(
        receiver: Receiver<ServerSocketEvent>,
        thread_handle: JoinHandle<()>,
    ) -> ReceiveHandler {
        ReceiveHandler {
            receiver,
            _thread_handle: thread_handle,
        }
    }

    // Returns an iterator that will block waiting for messages from the receiver but which will never panic!.
    // It will return None when the channel has hung up.
    pub fn iter(&self) -> Iter<'_, ServerSocketEvent> {
        self.receiver.iter()
    }
}
