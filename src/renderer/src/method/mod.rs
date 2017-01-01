pub mod forward;
pub mod deferred;

use gfx;

/// A `Method` is an implemnatnion of a Pass
pub trait Method<A, T, R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          A: ::Pass,
          T: ::Framebuffer
{
    fn apply(&self, arg: &A, target: &T, scene: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>);
}
