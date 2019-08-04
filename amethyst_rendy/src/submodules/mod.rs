//! Various helpers and implementations for sub functions of render passes.
mod environment;
mod flat_environment;
mod material;
mod skinning;
mod texture;
mod uniform;
mod vertex;

pub mod gather;

#[doc(no_inline)]
pub use environment::*;
#[doc(no_inline)]
pub use flat_environment::*;
#[doc(no_inline)]
pub use material::*;
#[doc(no_inline)]
pub use skinning::*;
#[doc(no_inline)]
pub use texture::*;
#[doc(no_inline)]
pub use uniform::*;
#[doc(no_inline)]
pub use vertex::*;
