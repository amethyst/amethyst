//! The send handler is used to communicate between a sending thread and it's owner.
//! The `SendHandler` communicates via `mpsc::channel`s.

use crate::{error::Result, server::ServerSocketEvent};
use std::{sync::mpsc::SyncSender, thread::JoinHandle};

/// Handler to access the internals of a sending socket.
pub struct SendHandler {
    /// handle that should be used fro reading the received packets on a given socket
    sender: SyncSender<ServerSocketEvent>,
    /// thread handle to the thread that sends packets
    _thread_handle: JoinHandle<()>,
}

impl SendHandler {
    /// Create a new receive handler by specifying:
    /// 1: handle that should be used fro reading the received packets on a given socket.
    /// 2: thread handle to the thread that sends packets.
    pub fn new(
        sender: SyncSender<ServerSocketEvent>,
        thread_handle: JoinHandle<()>,
    ) -> SendHandler {
        SendHandler {
            sender,
            _thread_handle: thread_handle,
        }
    }

    /// Send an event on the internal channel, by doing this it will be scheduled for sending.
    pub fn send(&self, event: ServerSocketEvent) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }

    /// Get the sending channel of this `SendHandler` to which you can send  
    pub fn get_sender(&self) -> &SyncSender<ServerSocketEvent> {
        &self.sender
    }
}
