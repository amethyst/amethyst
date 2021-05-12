use amethyst::{
    assets::LoaderBundle,
    core::transform::TransformBundle,
    ecs::*,
    renderer::{
        plugins::{RenderPbr3D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    Application, GameData, SimpleState, SimpleTrans, StateData, Trans,
};

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        example_utils::build_mesh_from_shape(world, resources);
        example_utils::build_spot_lights(world, resources);
        example_utils::initialize_camera(world);
    }
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let StateData {
            world: _,
            resources: _,
            ..
        } = data;
        let mut query = <(Entity,)>::query();
        let entities: Vec<Entity> = query.iter(data.world).map(|(ent,)| *ent).collect();
        for entity in entities {
            if let Some(_entry) = data.world.entry(entity) {
                //log::info!("{:?}: {:?}", entity, entry.archetype());
            }
        }
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut dispatcher = DispatcherBuilder::default();

    dispatcher
        // Loader bundle is needed for Rendering
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle::default())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderPbr3D::default()),
        );
    let game = Application::new(assets_dir, Example, dispatcher)?;
    game.run();
    Ok(())
}

/// Isolation of different functions that creates
/// components for the spotlights
mod example_utils {
    use amethyst::{
        assets::{DefaultLoader, Handle, Loader},
        core::{math::Vector3, num::FloatConst, transform::TransformValues, Transform},
        ecs::{Resources, World},
        renderer::{
            light::{Light, SpotLight},
            loaders::load_from_srgba,
            palette::{Srgb, Srgba},
            rendy::mesh::{Normal, Position, Tangent, TexCoord},
            shape::Shape,
            types::{MeshData, TextureData},
            Camera, Material, MaterialDefaults, Mesh, Texture,
        },
    };

    pub fn build_mesh_from_shape(world: &mut World, resources: &mut Resources) {
        let loader = resources.get::<DefaultLoader>().unwrap();
        let mesh_handle = {
            loader.load_from_data::<Mesh, (), MeshData>(
                Shape::Plane(None)
                    .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(Some((
                        1., 1., 1.,
                    )))
                    .into(),
                (),
                &resources.get().unwrap(),
            )
        };

        let mesh_transform = Transform::from(TransformValues::new(
            [0.0; 3],
            [0.0, 1.0, 0.0, 0.0],
            [4.0, 2.0, 1.0],
        ));

        let albedo: Handle<Texture> = {
            loader.load_from_data::<Texture, (), TextureData>(
                load_from_srgba(Srgba::new(1.0, 1.0, 1.0, 1.0)).into(),
                (),
                &resources.get().unwrap(),
            )
        };

        let mtl: Handle<Material> = {
            let mat_defaults = resources.get::<MaterialDefaults>().unwrap().0.clone();

            loader.load_from_data(
                Material {
                    albedo,
                    ..mat_defaults
                },
                (),
                &resources.get().unwrap(),
            )
        };

        world.push((mesh_handle, mesh_transform, mtl));
    }

    pub fn build_spot_lights(world: &mut World, _resources: &mut Resources) {
        let spotlight_1 = SpotLight {
            intensity: 3.0,
            color: Srgb::new(1.5, 0.0, 0.0),
            angle: 50.0,
            range: 4.0,
            smoothness: 0.0,
            direction: Vector3::new(0.0, 0.0, 1.0),
        };

        let mut spotlight_1_transform = Transform::default();
        spotlight_1_transform.set_translation_xyz(3.0, 0.0, -2.0);

        let spotlight_2 = SpotLight {
            intensity: 3.0,
            color: Srgb::new(0.0, 1.0, 0.0),
            angle: 30.0,
            range: 4.0,
            smoothness: 0.0,
            direction: Vector3::new(0.0, 0.0, 1.0),
        };

        let mut spotlight_2_transform = Transform::default();
        spotlight_2_transform.set_translation_xyz(1.25, 0.0, -2.0);

        let spotlight_3 = SpotLight {
            intensity: 3.0,
            color: Srgb::new(0.0, 0.0, 1.0),
            angle: 30.0,
            range: 4.0,
            smoothness: 0.0,
            direction: Vector3::new(0.0, 0.0, 1.0),
        };

        let mut spotlight_3_transform = Transform::default();
        spotlight_3_transform.set_translation_xyz(-1.25, 0.0, -2.0);

        let spotlight_4 = SpotLight {
            intensity: 2.0,
            color: Srgb::new(1.0, 1.0, 0.0),
            angle: 15.0,
            range: 10.0,
            smoothness: 0.8,
            direction: Vector3::new(1.0, -0.4, 0.4),
        };
        let mut spotlight_4_transform = Transform::default();
        spotlight_4_transform.set_translation_xyz(-5.0, 2., -1.5);

        world.extend(vec![
            (Light::Spot(spotlight_1), spotlight_1_transform),
            (Light::Spot(spotlight_2), spotlight_2_transform),
            (Light::Spot(spotlight_3), spotlight_3_transform),
            (Light::Spot(spotlight_4), spotlight_4_transform),
        ]);
    }

    pub fn initialize_camera(world: &mut World) {
        let camera_transform = Transform::from(TransformValues::new(
            [0.0, 0.0, -4.0],
            [0.0, 1.0, 0.0, 0.0],
            [1.0; 3],
        ));

        world.push((
            camera_transform,
            Camera::perspective(1.3, f32::FRAC_PI_3(), 0.1),
        ));
    }
}
