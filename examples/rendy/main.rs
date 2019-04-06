//! Displays spheres with physically based materials.
//!
use amethyst::{
    assets::AssetLoaderSystemData,
    core::{
        ecs::{
            Component, DenseVecStorage, Join, Read, ReadExpect, ReadStorage, Resources, System,
            SystemData, WriteStorage,
        },
        math::{Unit, UnitQuaternion, Vector3},
        Time, Transform, TransformBundle,
    },
    prelude::*,
    utils::{application_root_dir, fps_counter::FPSCounterBundle},
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

struct Orbit {
    axis: Unit<Vector3<f32>>,
    time_scale: f32,
    center: Vector3<f32>,
    radius: f32,
}

impl Component for Orbit {
    type Storage = DenseVecStorage<Self>;
}

struct OrbitSystem;

impl<'a> System<'a> for OrbitSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadStorage<'a, Orbit>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (time, orbits, mut transforms): Self::SystemData) {
        for (orbit, transform) in (&orbits, &mut transforms).join() {
            let angle = time.absolute_time_seconds() as f32 * orbit.time_scale;
            let cross = orbit.axis.cross(&Vector3::z()).normalize() * orbit.radius;
            let rot = UnitQuaternion::from_axis_angle(&orbit.axis, angle);
            let final_pos = (rot * cross) + orbit.center;
            transform.set_translation(final_pos);
        }
    }
}

struct CameraCorrectionSystem {
    last_aspect: f32,
}

impl CameraCorrectionSystem {
    pub fn new() -> Self {
        Self { last_aspect: 0.0 }
    }
}

impl<'a> System<'a> for CameraCorrectionSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        ReadExpect<'a, ActiveCamera>,
        WriteStorage<'a, Camera>,
    );

    fn run(&mut self, (dimensions, active_cam, mut cameras): Self::SystemData) {
        let current_aspect = dimensions.aspect_ratio();

        if current_aspect != self.last_aspect {
            self.last_aspect = current_aspect;

            let camera = cameras.get_mut(active_cam.entity).unwrap();
            *camera = Camera::from(Projection::perspective(
                current_aspect,
                std::f32::consts::FRAC_PI_3,
            ));
        }
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
        const NUM_ROWS: usize = 30;
        const NUM_COLS: usize = 30;

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
                        time_scale: 5.0 + y + 0.1 * x,
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
        .with(OrbitSystem, "orbit", &[])
        .with(CameraCorrectionSystem::new(), "cam", &[])
        .with_bundle(TransformBundle::new().with_dep(&["orbit"]))?
        .with_bundle(FPSCounterBundle::default())?
        .with_thread_local(EventsLoopSystem::new(event_loop))
        .with_thread_local(window_system)
        .with_thread_local(RendererSystem::<DefaultBackend, _>::new(ExampleGraph::new()));

    let mut game = Application::new(&resources, Example::<DefaultBackend>::new(), game_data)?;
    game.run();
    Ok(())
}

struct ExampleGraph {
    last_dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

impl ExampleGraph {
    pub fn new() -> Self {
        Self {
            last_dimensions: None,
            dirty: true,
        }
    }
}

impl<B: Backend> GraphCreator<B> for ExampleGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.last_dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.last_dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    fn builder(&mut self, factory: &mut Factory<B>, res: &Resources) -> GraphBuilder<B, Resources> {
        self.dirty = false;

        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);
        use amethyst_rendy::{
            pass::DrawPbmDesc,
            rendy::{
                graph::{
                    present::PresentNode,
                    render::{RenderGroupBuilder, RenderGroupDesc},
                    GraphBuilder,
                },
                hal::{
                    command::{ClearDepthStencil, ClearValue},
                    format::Format,
                    pso,
                },
                memory::MemoryUsageValue,
            },
        };

        let surface = factory.create_surface(window.clone());

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
            DrawPbmDesc::default()
                .with_vertex_skinning()
                .with_transparency(
                    pso::ColorBlendDesc(pso::ColorMask::ALL, pso::BlendState::ALPHA),
                    None,
                )
                .builder()
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
