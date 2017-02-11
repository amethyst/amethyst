pub mod forward;
pub mod deferred;

use std;
use gfx;
use mopa;

/// A `Pass` is an implementation of a Pass
pub trait Pass<R>: Sync
    where R: gfx::Resources
{
    /// The argument required for the Pass
    type Arg: ::PassDescription;
    /// The render Target
    type Target: ::Target;

    /// encode the pass into the encoder using the supplied argument
    /// frame and render target
    fn apply<C>(&self,
                arg: &Self::Arg,
                target: &Self::Target,
                pipeline: &::Pipeline,
                fragments: &[::Fragment<R>],
                scene: &::Scene,
                encoder: &mut gfx::Encoder<R, C>) where C: gfx::CommandBuffer<R>;
}

#[derive(Clone, Debug)]
/// Render the scene as a wireframe
pub struct Wireframe {
    /// The Camera to use
    pub camera: String,
    /// The scene to use
    pub scene: String,
}
impl PassDescription for Wireframe {}

impl Wireframe {
    /// Create a boxed Description of the Writeframe
    pub fn new<A, B>(camera: A, scene: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(Wireframe {
            camera: String::from(camera),
            scene: String::from(scene),
        })
    }
}

#[derive(Clone, Debug)]
/// Render into the target without any shading applied
pub struct DrawFlat {
    /// The Camera to use
    pub camera: String,
    /// The scene to use
    pub scene: String,
}
impl PassDescription for DrawFlat {}

impl DrawFlat {
    /// Create a Boxed DrawFlat
    pub fn new<A, B>(camera: A, scene: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(DrawFlat {
            camera: String::from(camera),
            scene: String::from(scene),
        })
    }
}

#[derive(Clone, Debug)]
/// Render only the depth layer leaving
/// all other Gbuffer layers unchanged
pub struct DepthPass {
    /// The Camera to use
    pub camera: String,
    /// The scene to use
    pub scene: String,
}

impl PassDescription for DepthPass {}

impl DepthPass {
    /// Create a Boxed DepthPass
    pub fn new<A, B>(camera: A, scene: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(DepthPass {
            camera: String::from(camera),
            scene: String::from(scene),
        })
    }
}

#[derive(Clone, Debug)]
/// Render into the target without a standard
/// ambient/diffuse shading
pub struct DrawShaded {
    /// The Camera to use
    pub camera: String,
    /// The scene to use
    pub scene: String,
}
impl PassDescription for DrawShaded {}

impl DrawShaded {
    /// Create a Boxed DrawShaded
    pub fn new<A, B>(camera: A, scene: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(DrawShaded {
            camera: String::from(camera),
            scene: String::from(scene),
        })
    }
}

#[derive(Clone, Debug)]
/// Blit a layer of the gbuffer into the target
pub struct BlitLayer {
    /// the gbuffer to blit from
    pub gbuffer: String,
    /// the layer of the buffer to blit from
    /// one of ka, kd or normal
    pub layer: String,
}
impl PassDescription for BlitLayer {}

impl BlitLayer {
    /// Create a boxed BlitLayer
    pub fn new<A, B>(gbuffer: A, layer: B) -> Box<PassDescription>
        where String: From<A> + From<B>
    {
        Box::new(BlitLayer {
            gbuffer: String::from(gbuffer),
            layer: String::from(layer),
        })
    }
}

#[derive(Clone, Debug)]
/// Do a lighting pass
pub struct Lighting {
    /// The Camera to use
    pub camera: String,
    /// The gbuffer to source the data
    pub gbuffer: String,
    /// the scene to get the lights from
    pub scene: String,
}
impl PassDescription for Lighting {}

impl Lighting {
    /// Box the Lighting Pass
    pub fn new<A, B, C>(camera: A, gbuffer: B, scene: C) -> Box<PassDescription>
        where String: From<A> + From<B> + From<C>
    {
        Box::new(Lighting {
            camera: String::from(camera),
            gbuffer: String::from(gbuffer),
            scene: String::from(scene),
        })
    }
}

/// Describes a render pass
pub trait PassDescription: mopa::Any + std::fmt::Debug + Sync {}
mopafy!(PassDescription);
