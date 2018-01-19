//!
//! Everything you want to do with descriptors but afraid to do it manually.
//!

mod pool;
mod bindings;
mod storage;

pub use self::bindings::{Binder, Binding, BindingsList, Layout, SetBinder, Uniform};
pub use self::pool::{DescriptorPool, DescriptorSet};
