//! ECS rendering bundle

use assets::{AssetStorage, Handle};
use core::bundle::{ECSBundle, Result};
use renderer::Config as DisplayConfig;
use renderer::Rgba;
use renderer::pipe::PipelineBuild;
use renderer::prelude::*;

use assets::{BoxedErr, Loader};
use ecs::{DispatcherBuilder, World};
use ecs::rendering::resources::{AmbientColor, ScreenDimensions, WindowMessages};
use ecs::rendering::systems::RenderSystem;
use ecs::transform::components::*;
use renderer::MaterialDefaults;

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
pub struct RenderBundle;

impl RenderBundle {
    /// Create a new render bundle
    pub fn new() -> Self {
        RenderBundle
    }
}

/// Create render system
pub fn create_render_system<P>(
    pipe: P,
    display_config: Option<DisplayConfig>,
) -> Result<RenderSystem<P::Pipeline>>
where
    P: PipelineBuild + Clone,
{
    let mut renderer = {
        let mut renderer = Renderer::build();

        if let Some(config) = display_config.to_owned() {
            renderer.with_config(config);
        }
        let renderer = renderer.build().map_err(BoxedErr::new)?;

        renderer
    };

    let pipe = renderer.create_pipe(pipe.clone()).map_err(BoxedErr::new)?;

    Ok(RenderSystem::new(pipe, renderer))
}


impl<'a, 'b> ECSBundle<'a, 'b> for RenderBundle {
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        use cgmath::Deg;
        use renderer::{Camera, Projection};

        let cam = Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        };

        world.add_resource(cam);
        world.add_resource(AmbientColor(Rgba::from([0.01; 3])));
        world.add_resource(WindowMessages::new());
        world.add_resource(ScreenDimensions::new(100, 100));
        world.add_resource(AssetStorage::<Mesh>::new());
        world.add_resource(AssetStorage::<Texture>::new());

        let mat = create_default_mat(world);
        world.add_resource(MaterialDefaults(mat));

        world.register::<Transform>();
        world.register::<Light>();
        world.register::<Material>();
        world.register::<Handle<Mesh>>();
        world.register::<Handle<Texture>>();

        Ok(builder)
    }
}

fn create_default_mat(world: &World) -> Material {
    let loader = world.read_resource::<Loader>();

    let albedo = [0.5, 0.5, 0.5, 1.0].into();
    let emission = [0.0; 4].into();
    let normal = [0.5, 0.5, 1.0, 1.0].into();
    let metallic = [0.0; 4].into();
    let roughness = [0.5; 4].into();
    let ambient_occlusion = [1.0; 4].into();
    let caveat = [1.0; 4].into();

    let tex_storage = world.read_resource();

    let albedo = loader.load_from_data(albedo, &tex_storage);
    let emission = loader.load_from_data(emission, &tex_storage);
    let normal = loader.load_from_data(normal, &tex_storage);
    let metallic = loader.load_from_data(metallic, &tex_storage);
    let roughness = loader.load_from_data(roughness, &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion, &tex_storage);
    let caveat = loader.load_from_data(caveat, &tex_storage);

    Material {
        albedo,
        emission,
        normal,
        metallic,
        roughness,
        ambient_occlusion,
        caveat,
    }
}
