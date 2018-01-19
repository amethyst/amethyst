//!
//! Top-level structure that encapsulates all pieces of rendering engine.
//! `HalConfig` to instantiate `HalBundle`.


mod build;
mod renderer;

use std::ops::Deref;

use core::bundle::ECSBundle;
use gfx_hal::Backend;
use shred::Resources;
use specs::{DispatcherBuilder, World};

use command::CommandCenter;
use epoch::CurrentEpoch;
use memory::Allocator;
use relevant::Relevant;
use upload::Uploader;

pub use hal::build::HalConfig;
pub use hal::renderer::{Renderer, RendererConfig};

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "No valid adapters queues found")]
    NoValidAdaptersFound,

    #[fail(display = "No compatible format found")]
    NoCompatibleFormat,
}

pub struct HalBundle<B: Backend> {
    pub device: B::Device,
    pub allocator: Allocator<B>,
    pub center: CommandCenter<B>,
    pub uploader: Uploader<B>,
    pub renderer: Option<Renderer<B>>,
    pub current: CurrentEpoch,
    relevant: Relevant,
}

impl<'a, 'b, B> ECSBundle<'a, 'b> for HalBundle<B>
where
    B: Backend,
{
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> ::core::Result<DispatcherBuilder<'a, 'b>> {
        use assets::{AssetStorage, Handle};
        use core::orientation::Orientation;
        use core::Transform;
        use camera::Camera;
        use light::{AmbientLight, Light};
        use material::{create_default_material, Material, MaterialDefaults};
        use mesh::Mesh;
        use resources::{WindowMessages, ScreenDimensions};
        use system::{ActiveGraph, RenderingSystem};
        use texture::Texture;

        let HalBundle {
            device,
            allocator,
            center,
            uploader,
            renderer,
            current,
            relevant,
        } = self;

        relevant.dispose();

        let hal = Hal {
            device: Device(device),
            allocator,
            uploader,
            current,
        };

        world.add_resource(hal);
        world.add_resource(ActiveGraph(None));
        world.add_resource(AmbientLight([0.01; 3]));
        world.add_resource(WindowMessages::new());
        world.add_resource(ScreenDimensions::new(0, 0));
        world.add_resource(AssetStorage::<Mesh<B>>::new());
        world.add_resource(AssetStorage::<Texture<B>>::new());
        world.add_resource(Orientation::default());

        let mat = create_default_material::<B>(world);
        world.add_resource(MaterialDefaults(mat));

        world.register::<Transform>();
        world.register::<Light>();
        world.register::<Material<B>>();
        world.register::<Handle<Mesh<B>>>();
        world.register::<Handle<Texture<B>>>();
        world.register::<Camera>();

        Ok(builder.add_thread_local(RenderingSystem {
            center,
            renderer,
        }))
    }
}

/// `Backend::Device` are actually `Send + Sync`. Except for OpenGL.
pub struct Device<B: Backend>(B::Device);
impl<B> Deref for Device<B>
where
    B: Backend,
{
    type Target = B::Device;
    fn deref(&self) -> &B::Device {
        &self.0
    }
}
unsafe impl<B> Send for Device<B> where B: Backend {}
unsafe impl<B> Sync for Device<B> where B: Backend {}

pub struct Hal<B: Backend> {
    pub device: Device<B>,
    pub allocator: Allocator<B>,
    pub uploader: Uploader<B>,
    pub current: CurrentEpoch,
}

impl<B> Hal<B>
where
    B: Backend,
{
    pub fn cleanup(&mut self, res: &Resources) {
        self.uploader.cleanup(&mut self.allocator);
        self.allocator.cleanup(&self.device, &self.current);
    }
}