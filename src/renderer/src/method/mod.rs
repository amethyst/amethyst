pub mod forward;
pub mod deferred;

use gfx;

/// A `Method` is an implemnatnion of a Pass
pub trait Method<R>
    where R: gfx::Resources,
{
    type Arg: ::Pass;
    type Target: ::Framebuffer;

    fn apply<C>(&self, arg: &Self::Arg, target: &Self::Target, scene: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>;
}
