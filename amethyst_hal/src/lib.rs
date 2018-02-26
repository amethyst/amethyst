//!

// #![deny(missing_docs)]
#![deny(unused_must_use)]

extern crate amethyst_assets;
extern crate amethyst_core as core;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate gfx_hal as hal;
extern crate gfx_memory as mem;
#[macro_use]
extern crate log;
extern crate shred;
extern crate smallvec;
extern crate specs;
extern crate winit;
extern crate xfg;

#[cfg(feature = "gfx-dx12")]
extern crate gfx_backend_dx12 as dx12;
#[cfg(not(any(feature = "gfx-vulkan", feature = "gfx-metal", feature = "gfx-dx12")))]
extern crate gfx_backend_empty as empty;
#[cfg(feature = "gfx-metal")]
extern crate gfx_backend_metal as metal;
#[cfg(feature = "gfx-vulkan")]
extern crate gfx_backend_vulkan as vulkan;

extern crate wavefront_obj;

use hal::Backend;
use mem::{Item, SmartBlock};
use shred::Resources;
use xfg::{Graph, GraphBuilder, Pass};

error_chain! {}

/// Buffer type used in engiene
pub type Buffer<B: Backend> = Item<B::Buffer, SmartBlock<B::Memory>>;

/// Image type used in engiene
pub type Image<B: Backend> = Item<B::Image, SmartBlock<B::Memory>>;

/// Boxed type of pass used in engine
pub type AmethystPass<B: Backend> = Box<for<'a> Pass<B, &'a Resources> + Send + Sync>;

/// Graph type used in engine.
pub type AmethystGraph<B: Backend> = Graph<B, Image<B>, AmethystPass<B>>;

/// GraphBuilder type used in engine.
pub type AmethystGraphBuilder<B: Backend> = GraphBuilder<AmethystPass<B>>;

mod assets;
pub mod backend;
pub mod bundle;
pub mod factory;
pub mod light;
pub mod mesh;
pub mod system;
pub mod utils;
pub mod vertex;

pub use bundle::RenderBundle;
pub use factory::Factory;
pub use system::RenderSystem;
