pub mod forward;
pub mod deferred;

use gfx;
use mopa;

/// A `Pass` is an implemnatnion of a Pass
pub trait Pass<R>
    where R: gfx::Resources,
{
    type Arg: ::PassDescription;
    type Target: ::Target;

    fn apply<C>(&self, arg: &Self::Arg, target: &Self::Target, scene: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>;
}

pub struct Clear {
    pub color: [f32; 4]
}
impl PassDescription for Clear {}

impl Clear {
    pub fn new(color: [f32; 4]) -> Box<PassDescription> {
        Box::new(Clear{
            color: color
        })
    }
}

pub struct Wireframe {
    pub camera: String,
    pub scene: String,
}
impl PassDescription for Wireframe {}

impl Wireframe {
    pub fn new<A, B>(camera: A, scene: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(Wireframe{
            camera: String::from(camera),
            scene: String::from(scene),
        })
    }
}

pub struct DrawNoShading {
    pub camera: String,
    pub scene: String,
}
impl PassDescription for DrawNoShading {}

impl DrawNoShading {
    pub fn new<A, B>(camera: A, scene: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(DrawNoShading{
            camera: String::from(camera),
            scene: String::from(scene),
        })
    }
}

pub struct BlitLayer {
    pub gbuffer: String,
    pub layer: String,
}
impl PassDescription for BlitLayer {}

impl BlitLayer {
    pub fn new<A, B>(gbuffer: A, layer: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(BlitLayer{
            gbuffer: String::from(gbuffer),
            layer: String::from(layer),
        })
    }
}

pub struct Lighting {
    pub camera: String,
    pub gbuffer: String,
    pub scene: String,
}
impl PassDescription for Lighting {}

impl Lighting {
    pub fn new<A, B, C>(camera: A, gbuffer: B, scene: C) -> Box<PassDescription>
        where String: From<A> + From<B> + From<C>
    {
        Box::new(Lighting{
            camera: String::from(camera),
            gbuffer: String::from(gbuffer),
            scene: String::from(scene),
        })
    }
}

/// Describes a render pass
pub trait PassDescription: mopa::Any {}
mopafy!(PassDescription);

