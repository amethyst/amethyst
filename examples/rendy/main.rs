//! Displays spheres with physically based materials.
//!
use amethyst::{
    assets::AssetLoaderSystemData,
    core::{
        ecs::{Resources, SystemData, ReadExpect},
        Transform, TransformBundle,
    },
    prelude::*,
    utils::application_root_dir,
    winit::{Window, EventsLoop},
    window::{WindowSystem, EventsLoopSystem},
};
use amethyst_rendy::{
    camera::{Camera, Projection},
    light::{Light, PointLight},
    mtl::{Material, MaterialDefaults},
    palette::{LinLuma, LinSrgb, Srgb},
    rendy::{
        mesh::PosNormTangTex,
        texture::palette::load_from_linear_rgba,
        graph::{GraphBuilder},
        hal::Backend,
        factory::Factory,
    },
    shape::Shape,
    system::RendererSystem,
    types::{Mesh, Texture, DefaultBackend},
};
use std::sync::Arc;
use std::marker::PhantomData;

struct Example<B: Backend>(PhantomData<B>);
impl<B: Backend> Example<B> {
    pub fn new() -> Self { Self(PhantomData) }
}

impl<B: Backend> SimpleState for Example<B> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        let mat_defaults = world.read_resource::<MaterialDefaults<B>>().0.clone();

        let (mesh, albedo) = {
            let mesh = world.exec(|loader: AssetLoaderSystemData<'_, Mesh<B>>| {
                loader.load_from_data(
                    Shape::Sphere(32, 32).generate::<Vec<PosNormTangTex>>(None),
                    (),
                )
            });
            let albedo = world.exec(|loader: AssetLoaderSystemData<'_, Texture<B>>| {
                loader.load_from_data(
                    load_from_linear_rgba(LinSrgb::new(1.0, 1.0, 1.0).into()),
                    (),
                )
            });

            (mesh, albedo)
        };

        println!("Create spheres");
        for i in 0..5 {
            for j in 0..5 {
                let roughness = 1.0f32 * (i as f32 / 4.0f32);
                let metallic = 1.0f32 * (j as f32 / 4.0f32);

                let mut pos = Transform::default();
                pos.set_translation_xyz(2.0f32 * (i - 2) as f32, 2.0f32 * (j - 2) as f32, 0.0);

                let (metallic, roughness) =
                    world.exec(|loader: AssetLoaderSystemData<'_, Texture<B>>| {
                        (
                            loader.load_from_data(
                                load_from_linear_rgba(LinLuma::new(metallic).into()),
                                (),
                            ),
                            loader.load_from_data(
                                load_from_linear_rgba(LinLuma::new(roughness).into()),
                                (),
                            ),
                        )
                    });

                let mtl = world.exec(|loader: AssetLoaderSystemData<'_, Material<B>>| {
                    loader.load_from_data(
                        Material {
                        albedo: albedo.clone(),
                        metallic,
                        roughness,
                        ..mat_defaults.clone()
                    },
                        (),
                    )
                });

                world
                    .create_entity()
                    .with(pos)
                    .with(mesh.clone())
                    .with(mtl)
                    .build();
            }
        }

        println!("Create lights");
        let light1: Light = PointLight {
            intensity: 6.0,
            color: Srgb::new(0.8, 0.0, 0.0),
            ..PointLight::default()
        }
        .into();

        let mut light1_transform = Transform::default();
        light1_transform.set_translation_xyz(6.0, 6.0, -6.0);

        let light2: Light = PointLight {
            intensity: 5.0,
            color: Srgb::new(0.0, 0.3, 0.7),
            ..PointLight::default()
        }
        .into();

        let mut light2_transform = Transform::default();
        light2_transform.set_translation_xyz(6.0, -6.0, -6.0);

        world
            .create_entity()
            .with(light1)
            .with(light1_transform)
            .build();

        world
            .create_entity()
            .with(light2)
            .with(light2_transform)
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -12.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        world
            .create_entity()
            .with(Camera::from(Projection::perspective(
                1.3,
                std::f32::consts::FRAC_PI_3,
            )))
            .with(transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let path = app_root.join("examples/rendy/resources/display_config.ron");
    let resources = app_root.join("examples/assets/");

    let event_loop = EventsLoop::new();

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_thread_local(WindowSystem::from_config_path(&event_loop, path))
        .with_thread_local(RendererSystem::<DefaultBackend, _>::new(build_graph))
        .with_thread_local(EventsLoopSystem::new(event_loop));

    let mut game = Application::new(&resources, Example::<DefaultBackend>::new(), game_data)?;
    game.run();
    Ok(())
}

fn build_graph<B: Backend>(
    factory: &mut Factory<B>,
    res: &mut Resources,
) -> GraphBuilder<B, Resources> {
    let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);
    use amethyst_rendy::{
        rendy::{
            graph::{
                present::PresentNode,
                render::{
                    SimpleGraphicsPipeline,
                    RenderGroupBuilder,
                },
                GraphBuilder,
            },
            memory::MemoryUsageValue,
            hal::{
                command::{ClearValue, ClearDepthStencil},
                format::Format,
            }
        },
        pass::DrawPbm
    };

    let surface = factory.create_surface(window.clone());
    // let aspect = surface.aspect();
    
    let mut graph_builder = GraphBuilder::new();

    let color = graph_builder.create_image(
        surface.kind(),
        1,
        factory.get_surface_format(&surface),
        MemoryUsageValue::Data,
        Some(ClearValue::Color(
            [1.0, 1.0, 1.0, 1.0].into(),
        )),
    );
    
    let depth = graph_builder.create_image(
        surface.kind(),
        1,
        Format::D16Unorm,
        MemoryUsageValue::Data,
        Some(ClearValue::DepthStencil(
            ClearDepthStencil(1.0, 0),
        )),
    );

    let pass = graph_builder.add_node(
        DrawPbm::builder()
            .into_subpass()
            .with_color(color)
            .with_depth_stencil(depth)
            .into_pass(),
    );

    let present_builder = PresentNode::builder(factory, surface, color)
        .with_dependency(pass);

    graph_builder.add_node(present_builder);

    graph_builder
}

