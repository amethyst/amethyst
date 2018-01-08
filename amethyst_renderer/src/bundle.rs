//! ECS rendering bundle

use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::bundle::{ECSBundle, Result};
use amethyst_core::orientation::Orientation;
use amethyst_core::transform::components::*;
use specs::{DispatcherBuilder, World};

use {AmbientColor, Camera, Light, Material, MaterialAnimation, MaterialDefaults, Mesh, Rgba,
     ScreenDimensions, SpriteSheetData, TexAnimationSystem, Texture, WindowMessages};

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
#[derive(Default)]
pub struct RenderBundle {
    // defaults to false, meaning Texture Animation is included by default
    without_texture_animation: bool,
}

impl RenderBundle {
    /// Create a new render bundle
    pub fn new() -> Self {
        Default::default()
    }

    /// Remove Texture Animation from bundle (included by default)
    pub fn without_texture_animation(mut self) -> Self {
        self.without_texture_animation = true;
        self
    }
}

impl<'a, 'b> ECSBundle<'a, 'b> for RenderBundle {
    fn build(
        self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(AmbientColor(Rgba::from([0.01; 3])));
        world.add_resource(WindowMessages::new());
        world.add_resource(ScreenDimensions::new(100, 100));
        world.add_resource(AssetStorage::<Mesh>::new());
        world.add_resource(AssetStorage::<Texture>::new());
        world.add_resource(Orientation::default());

        let mat = create_default_mat(world);
        world.add_resource(MaterialDefaults(mat));

        world.register::<Transform>();
        world.register::<Light>();
        world.register::<Material>();
        world.register::<Handle<Mesh>>();
        world.register::<Handle<Texture>>();
        world.register::<Camera>();

        if !self.without_texture_animation {
            world.add_resource(AssetStorage::<SpriteSheetData>::new());
            world.register::<MaterialAnimation>();
            world.register::<Handle<SpriteSheetData>>();
            builder = builder.add(TexAnimationSystem {}, "texture_animation_system", &[])
        };

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

    let albedo = loader.load_from_data(albedo, (), &tex_storage);
    let emission = loader.load_from_data(emission, (), &tex_storage);
    let normal = loader.load_from_data(normal, (), &tex_storage);
    let metallic = loader.load_from_data(metallic, (), &tex_storage);
    let roughness = loader.load_from_data(roughness, (), &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion, (), &tex_storage);
    let caveat = loader.load_from_data(caveat, (), &tex_storage);

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
