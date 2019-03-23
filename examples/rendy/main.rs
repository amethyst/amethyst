//! Displays spheres with physically based materials.
//!
use amethyst::{
    assets::AssetLoaderSystemData,
    core::{
        ecs::{ReadExpect, Resources, SystemData},
        Transform, TransformBundle,
    },
    prelude::*,
    utils::application_root_dir,
    window::{EventsLoopSystem, ScreenDimensions, WindowSystem},
    winit::{EventsLoop, Window},
};
use amethyst_rendy::{
    camera::{ActiveCamera, Camera, Projection},
    light::{Light, PointLight},
    mtl::{Material, MaterialDefaults},
    palette::{LinLuma, LinSrgb, Srgb},
    rendy::{
        factory::Factory, graph::GraphBuilder, hal::Backend, mesh::PosNormTangTex,
        texture::palette::load_from_linear_rgba,
    },
    resources::Tint,
    shape::Shape,
    system::{GraphCreator, RendererSystem},
    types::{DefaultBackend, Mesh, Texture},
};
use std::{marker::PhantomData, sync::Arc};

struct Example<B: Backend>(PhantomData<B>);
impl<B: Backend> Example<B> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
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
        const NUM_ROWS: usize = 100;
        const NUM_COLS: usize = 100;

        let mut mtls = Vec::with_capacity(100);

        for i in 0..10 {
            for j in 0..10 {
                let roughness = i as f32 / 9.0;
                let metallic = j as f32 / 9.0;
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
                mtls.push(mtl);
            }
        }

        for i in 0..NUM_COLS {
            for j in 0..NUM_ROWS {
                let x = i as f32 / (NUM_COLS - 1) as f32;
                let y = j as f32 / (NUM_ROWS - 1) as f32;

                let center = Vector3::new(10.0 * (x - 0.5), 10.0 * (y - 0.5), 0.0);

                let mut pos = Transform::default();
                pos.set_translation(center);
                pos.set_scale(0.2, 0.2, 0.2);

                let mut builder = world
                    .create_entity()
                    .with(pos)
                    .with(mesh.clone())
                    .with(mtls[(j + i) % mtls.len()].clone())
                    .with(Orbit {
                        axis: Unit::new_normalize(Vector3::y()),
                        time_scale: 5.0 + y,
                        center,
                        radius: 0.2,
                    });
                
                // add some visible tint pattern
                if i > 10 && j > 10 && i < NUM_COLS - 10 && j < NUM_ROWS - 10 {
                    let xor_x = i - 10;
                    let xor_y = j - 10;
                    let c = ((xor_x ^ xor_y) & 0xFF) as f32 / 255.0;
                    builder = builder.with(Tint(Srgb::new(c, c, c).into()));
                }

                builder.build();
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
            .with(Orbit {
                axis: Unit::new_normalize(Vector3::x()),
                time_scale: 2.0,
                center: Vector3::new(6.0, -6.0, -6.0),
                radius: 5.0,
            })
            .build();

        world
            .create_entity()
            .with(light2)
            .with(light2_transform)
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -12.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        let camera = world
            .create_entity()
            .with(Camera::from(Projection::perspective(
                1.3,
                std::f32::consts::FRAC_PI_3,
            )))
            .with(transform)
            .build();

        world.add_resource(ActiveCamera { entity: camera });
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(amethyst::LoggerConfig {
        log_file: Some("rendy_example.log".into()),
        level_filter: log::LevelFilter::Error,
        ..Default::default()
    })
    .level_for("amethyst_utils::fps_counter", log::LevelFilter::Debug)
    // .level_for("rendy_factory", log::LevelFilter::Trace)
    // .level_for("rendy_resource", log::LevelFilter::Trace)
    // .level_for("rendy_descriptor", log::LevelFilter::Trace)
    .start();

    let app_root = application_root_dir()?;

    let path = app_root.join("examples/rendy/resources/display_config.ron");
    let resources = app_root.join("examples/assets/");

    let event_loop = EventsLoop::new();

    let window_system = WindowSystem::from_config_path(&event_loop, path);

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_thread_local(EventsLoopSystem::new(event_loop))
        .with_thread_local(window_system)
        .with_thread_local(RendererSystem::<DefaultBackend, _>::new(ExampleGraph));

    let mut game = Application::new(&resources, Example::<DefaultBackend>::new(), game_data)?;
    game.run();
    Ok(())
}

struct ExampleGraph;

impl<B: Backend> GraphCreator<B> for ExampleGraph {
    type GraphDeps = Option<ScreenDimensions>;
    fn dependencies(&self, res: &Resources) -> Self::GraphDeps {
        res.try_fetch::<ScreenDimensions>().map(|d| d.clone())
    }

    fn builder(
        &self,
        factory: &mut Factory<B>,
        res: &Resources,
        _deps: &Self::GraphDeps,
    ) -> GraphBuilder<B, Resources> {
        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);
        use amethyst_rendy::{
            pass::DrawPbm,
            rendy::{
                graph::{
                    present::PresentNode,
                    render::{RenderGroupBuilder, SimpleGraphicsPipeline},
                    GraphBuilder,
                },
                hal::{
                    command::{ClearDepthStencil, ClearValue},
                    format::Format,
                },
                memory::MemoryUsageValue,
            },
        };

        let surface = factory.create_surface(window.clone());
        // let aspect = surface.aspect();

        let mut graph_builder = GraphBuilder::new();

        let color = graph_builder.create_image(
            surface.kind(),
            1,
            factory.get_surface_format(&surface),
            MemoryUsageValue::Data,
            Some(ClearValue::Color([1.0, 1.0, 1.0, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            surface.kind(),
            1,
            Format::D16Unorm,
            MemoryUsageValue::Data,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let pass = graph_builder.add_node(
            DrawPbm::builder()
                .into_subpass()
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let present_builder = PresentNode::builder(factory, surface, color).with_dependency(pass);

        graph_builder.add_node(present_builder);

        graph_builder
    }
}
