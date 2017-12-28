//!
//! Everything you want to do with descriptors but afraid to do it manually.
//! 


mod pool;
mod bindings;

pub use self::pool::{DescriptorPool, DescriptorSet};
pub use self::bindings::{Binding, Uniform, BindingsList, Layout, Binder, SetBinder};