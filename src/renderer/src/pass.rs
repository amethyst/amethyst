use mopa;

pub struct Clear {
    pub color: [f32; 4]
}
impl Pass for Clear {}

impl Clear {
    pub fn new(color: [f32; 4]) -> Box<Pass> {
        Box::new(Clear{
            color: color
        })
    }
}

pub struct Wireframe {
    pub camera: String,
    pub scene: String,
}
impl Pass for Wireframe {}

impl Wireframe {
    pub fn new<A, B>(camera: A, scene: B) -> Box<Pass>
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
impl Pass for DrawNoShading {}

impl DrawNoShading {
    pub fn new<A, B>(camera: A, scene: B) -> Box<Pass>
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
impl Pass for BlitLayer {}

impl BlitLayer {
    pub fn new<A, B>(gbuffer: A, layer: B) -> Box<Pass>
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
impl Pass for Lighting {}

impl Lighting {
    pub fn new<A, B, C>(camera: A, gbuffer: B, scene: C) -> Box<Pass>
        where String: From<A> + From<B> + From<C>
    {
        Box::new(Lighting{
            camera: String::from(camera),
            gbuffer: String::from(gbuffer),
            scene: String::from(scene),
        })
    }
}

pub trait Pass: mopa::Any {}
mopafy!(Pass);

