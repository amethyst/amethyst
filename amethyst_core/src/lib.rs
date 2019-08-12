//! A collection of structures and functions useful across the entire amethyst project.
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

#[macro_use]
extern crate getset;
#[macro_use]
extern crate derive_new;

pub use alga;
pub use approx;
pub use nalgebra as math;
pub use num_traits as num;
pub use shred;
pub use shrev;
pub use specs as ecs;

use rayon;

use std::sync::Arc;

pub use crate::{
    bundle::SystemBundle,
    event::EventReader,
    system_ext::{Pausable, SystemExt},
    timing::*,
    transform::*,
};

pub use self::{
    axis::{Axis2, Axis3},
    hidden::{Hidden, HiddenPropagate},
    hide_system::HideHierarchySystem,
    named::{Named, WithNamed},
};

pub mod bundle;
pub mod frame_limiter;
pub mod timing;
pub mod transform;

mod axis;
mod event;
mod hidden;
mod hide_system;
mod named;
mod system_ext;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
// pub type ArcThreadPool = Arc<rayon::ThreadPool>;

/// `Send + Sync` wrapper for objects that may not be `Send` or `Sync`.
/// It panics if inner value get accessed from any thread
/// except one where wrapper was created.
pub struct St<T> {
    value: T,
    thread: std::thread::ThreadId,
}

impl<T> St<T> {
    pub fn new(value: T) -> Self {
        St {
            value,
            thread: std::thread::current().id(),
        }
    }

    pub fn get_ref(st: &Self) -> &T {
        assert_eq!(std::thread::current().id(), st.thread);
        &st.value
    }

    pub fn get_mut(st: &mut Self) -> &mut T {
        assert_eq!(std::thread::current().id(), st.thread);
        &mut st.value
    }

    pub fn into_inner(st: Self) -> T {
        st.value
    }
}

impl<T> std::ops::Deref for St<T> {
    type Target = T;

    fn deref(&self) -> &T {
        Self::get_ref(self)
    }
}

impl<T> std::ops::DerefMut for St<T> {
    fn deref_mut(&mut self) -> &mut T {
        Self::get_mut(self)
    }
}

unsafe impl<T> Send for St<T> {}
unsafe impl<T> Sync for St<T> {}

pub type EventLoopRes = St<winit::event_loop::EventLoop<()>>;
pub type WindowRes = St<winit::window::Window>;

