use crate::core::specs::World;
use crossbeam_channel::{Receiver, Sender};

/// The type of a callback.
/// This is meant to be created from within asynchonous functions (`Future` for example).
/// See `CallbackQueue` for more details.
pub type Callback = Box<dyn Fn(&mut World) + Send>;

/// A simple `Callback` queue.
/// Using the `Sender` you can get using the `send_handle` method, you
/// can add functions modifying `World` from an asynchronous context.
/// Those callbacks will be ran sequentially without preserving ordering.
/// # Example
/// ```rust,ignore
/// // First, get a `Sender` handle.
/// let handle = world.read_resource::<CallbackQueue>().send_handle();
/// // Then, create your asynchronous context (Future, Callback-based library, etc..)
/// let future = ...;
/// // Finally, use that handle inside of the asynchronous context to run code that can affect `World`.
/// future.on_complete(move || {
///     handle.send(|mut world| world.create_entity().build()).expect("Failed to add Callback to CallbackQueue.");
/// });
/// ```
pub struct CallbackQueue {
    sender: Sender<Callback>,
    pub(crate) receiver: Receiver<Callback>,
}

impl CallbackQueue {
    /// Creates a new `CallbackQueue`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new handle that allows sending `Callback`s to the `CallbackQueue`.
    pub fn send_handle(&self) -> Sender<Callback> {
        self.sender.clone()
    }
}

impl Default for CallbackQueue {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        CallbackQueue { sender, receiver }
    }
}
