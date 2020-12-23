//! A collection of structures and functions useful across the entire amethyst project.
#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

#[cfg(all(target_os = "emscripten", not(no_threading)))]
compile_error!("the cfg flag \"no_threading\" is required when building for emscripten");

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource.
pub type ArcThreadPool = std::sync::Arc<rayon::ThreadPool>;

pub use core::fmt; //FIXME https://github.com/amethyst/amethyst/issues/2478

pub use approx;
pub use nalgebra as math;
pub use num_traits as num;
pub use shrev;
pub use simba as simd;

pub use self::{
    axis::{Axis2, Axis3},
    event::EventReader,
    hidden::{Hidden, HiddenPropagate},
    named::Named,
    shrev::EventChannel,
    timing::*,
};

/// legion ECS reexported with some convenience types.
pub mod ecs {
    pub use legion::{
        systems::{CommandBuffer, ParallelRunnable, Resource, Runnable},
        world::SubWorld,
        *,
    };

    pub use crate::dispatcher::{Dispatcher, DispatcherBuilder, System, SystemBundle};
}

/// Dispatcher module.
pub mod dispatcher;

/// The frame limiter module.
pub mod frame_limiter;

/// The geometry module.
pub mod geometry;

/// The timing module.
pub mod timing;

/// The transformation module.
pub mod transform;

/// The hide hierarchy system
pub mod hide_hierarchy_system;

mod axis;
mod event;
mod hidden;
mod named;
pub mod system_ext;
