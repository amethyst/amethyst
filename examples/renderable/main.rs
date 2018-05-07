//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;

use amethyst::{Application, Error, GameData, GameDataBuilder, State, StateData, Trans};
use amethyst::assets::{Completion, HotReloadBundle, Loader, ProgressCounter};
use amethyst::config::Config;
use amethyst::core::cgmath::{Array, Deg, Euler, Quaternion, Rad, Rotation, Rotation3, Vector3};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::timing::Time;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::{Component, Entity, Join, Read, ReadStorage, System, World,
                             WriteExpect, WriteStorage};
use amethyst::ecs::storage::NullStorage;
use amethyst::input::InputBundle;
use amethyst::renderer::{AmbientColor, Camera, DirectionalLight, DisplayConfig, DrawShaded,
                         ElementState, Event, KeyboardInput, Light, Material, MaterialDefaults,
                         MeshHandle, ObjFormat, Pipeline, PngFormat, PointLight, PosNormTex,
                         Projection, RenderBundle, Rgba, Stage, VirtualKeyCode, WindowEvent};
use amethyst::ui::{Anchor, Anchored, DrawUi, FontHandle, TtfFormat, UiBundle, UiText, UiTransform};
use amethyst::utils::fps_counter::{FPSCounter, FPSCounterBundle};

struct DemoState {
    light_angle: f32,
    light_color: [f32; 4],
    ambient_light: bool,
    point_light: bool,
    directional_light: bool,
    camera_angle: f32,
    fps_display: Entity,
    #[allow(dead_code)]
    pipeline_forward: bool, // TODO
}

struct ExampleSystem;

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (
        WriteStorage<'a, Light>,
        Read<'a, Time>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
        WriteExpect<'a, DemoState>,
        WriteStorage<'a, UiText>,
        Read<'a, FPSCounter>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut lights, time, camera, mut transforms, mut state, mut ui_text, fps_counter) = data;
        let light_angular_velocity = -1.0;
        let light_orbit_radius = 15.0;
        let light_z = 6.0;

        let camera_angular_velocity = 0.1;

        state.light_angle += light_angular_velocity * time.delta_seconds();
        state.camera_angle += camera_angular_velocity * time.delta_seconds();

        let delta_rot =
            Quaternion::from_angle_z(Rad(camera_angular_velocity * time.delta_seconds()));
        for (_, transform) in (&camera, &mut transforms).join() {
            // rotate the camera, using the origin as a pivot point
            transform.translation = delta_rot.rotate_vector(transform.translation);
            // add the delta rotation for the frame to the total rotation (quaternion multiplication
            // is the same as rotational addition)
            transform.rotation = (delta_rot * Quaternion::from(transform.rotation)).into();
        }

        for point_light in (&mut lights).join().filter_map(|light| {
            if let Light::Point(ref mut point_light) = *light {
                Some(point_light)
            } else {
                None
            }
        }) {
            point_light.center[0] = light_orbit_radius * state.light_angle.cos();
            point_light.center[1] = light_orbit_radius * state.light_angle.sin();
            point_light.center[2] = light_z;

            point_light.color = state.light_color.into();
        }

        if let Some(fps_display) = ui_text.get_mut(state.fps_display) {
            if time.frame_number() % 20 == 0 {
                let fps = fps_counter.sampled_fps();
                fps_display.text = format!("FPS: {:.*}", 2, fps);
            }
        }
    }
}

#[derive(Default)]
struct Loading {
    progress: Option<ProgressCounter>,
}
struct Example;

#[derive(Default)]
struct LoadTag;

impl Component for LoadTag {
    type Storage = NullStorage<Self>;
}

impl<'a, 'b> State<GameData<'a, 'b>> for Loading {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world.register::<LoadTag>();
        let (progress, assets) = load_assets(&data.world);
        self.progress = Some(progress);
        let fps_display = data.world
            .create_entity()
            .with(UiTransform::new(
                "fps".to_string(),
                100.,
                25.,
                1.,
                200.,
                50.,
                0,
            ))
            .with(UiText::new(
                assets.font.clone(),
                "N/A".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                25.,
            ))
            .with(Anchored::new(Anchor::TopLeft))
            .build();

        data.world
            .create_entity()
            .with(UiTransform::new(
                "fps".to_string(),
                0.,
                0.,
                1.,
                200.,
                50.,
                0,
            ))
            .with(UiText::new(
                assets.font.clone(),
                "Loading".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                25.,
            ))
            .with(Anchored::new(Anchor::Middle))
            .with(LoadTag)
            .build();

        data.world.add_resource(assets);
        data.world.add_resource::<DemoState>(DemoState {
            light_angle: 0.0,
            light_color: [1.0; 4],
            ambient_light: true,
            point_light: true,
            directional_light: true,
            camera_angle: 0.0,
            fps_display,
            pipeline_forward: true,
        });
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        match self.progress.as_ref().unwrap().complete() {
            Completion::Failed => {
                println!("Failed loading assets");
                Trans::Quit
            }
            Completion::Complete => {
                println!("Assets loaded, swapping state");
                let entities = (
                    &*data.world.entities(),
                    &data.world.read_storage::<LoadTag>(),
                ).join()
                    .map(|(e, _)| e)
                    .collect::<Vec<_>>();
                if let Err(err) = data.world.delete_entities(&entities) {
                    println!("Failed deleting load state entities: {}", err);
                }
                Trans::Switch(Box::new(Example))
            }
            Completion::Loading => Trans::None,
        }
    }
}

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        initialise_camera(world);

        let assets = world.read_resource::<Assets>().clone();

        // Add teapot and lid to scene
        for mesh in vec![assets.lid.clone(), assets.teapot.clone()] {
            let mut trans = Transform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = Vector3::new(5.0, 5.0, 0.0);

            world
                .create_entity()
                .with(mesh)
                .with(assets.red.clone())
                .with(trans)
                .with(GlobalTransform::default())
                .build();
        }

        // Add cube to scene
        let mut trans = Transform::default();
        trans.translation = Vector3::new(5.0, -5.0, 2.0);
        trans.scale = [2.0; 3].into();

        world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.logo.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();

        // Add cone to scene
        let mut trans = Transform::default();
        trans.translation = Vector3::new(-5.0, 5.0, 0.0);
        trans.scale = [2.0; 3].into();

        world
            .create_entity()
            .with(assets.cone.clone())
            .with(assets.white.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();

        // Add custom cube object to scene
        let mut trans = Transform::default();
        trans.translation = Vector3::new(-5.0, -5.0, 1.0);
        world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.red.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();

        // Create base rectangle as floor
        let mut trans = Transform::default();
        trans.scale = Vector3::from_value(10.);

        world
            .create_entity()
            .with(assets.rectangle.clone())
            .with(assets.white.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();

        let light: Light = PointLight {
            color: [1.0, 1.0, 0.0].into(),
            intensity: 50.0,
            ..PointLight::default()
        }.into();

        // Add lights to scene
        world.create_entity().with(light).build();

        let light: Light = DirectionalLight {
            color: [0.2; 4].into(),
            direction: [-1.0; 3],
        }.into();

        world.create_entity().with(light).build();
        world.add_resource(AmbientColor(Rgba::from([0.01; 3])));
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        let w = data.world;
        // Exit if user hits Escape or closes the window
        let mut state = w.write_resource::<DemoState>();

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        match virtual_keycode {
                            Some(VirtualKeyCode::Escape) => return Trans::Quit,
                            Some(VirtualKeyCode::Space) => {
                                // TODO: figure out how to change pipeline
                                /*if state.pipeline_forward {
                                    state.pipeline_forward = false;
                                    set_pipeline_state(pipe, false);
                                } else {
                                    state.pipeline_forward = true;
                                    set_pipeline_state(pipe, true);
                                }*/
                            }
                            Some(VirtualKeyCode::R) => {
                                state.light_color = [0.8, 0.2, 0.2, 1.0];
                            }
                            Some(VirtualKeyCode::G) => {
                                state.light_color = [0.2, 0.8, 0.2, 1.0];
                            }
                            Some(VirtualKeyCode::B) => {
                                state.light_color = [0.2, 0.2, 0.8, 1.0];
                            }
                            Some(VirtualKeyCode::W) => {
                                state.light_color = [1.0, 1.0, 1.0, 1.0];
                            }
                            Some(VirtualKeyCode::A) => {
                                let mut color = w.write_resource::<AmbientColor>();
                                if state.ambient_light {
                                    state.ambient_light = false;
                                    color.0 = [0.0; 3].into();
                                } else {
                                    state.ambient_light = true;
                                    color.0 = [0.01; 3].into();
                                }
                            }
                            Some(VirtualKeyCode::D) => {
                                let mut lights = w.write_storage::<Light>();

                                if state.directional_light {
                                    state.directional_light = false;
                                    for light in (&mut lights).join() {
                                        if let Light::Directional(ref mut d) = *light {
                                            d.color = [0.0; 4].into();
                                        }
                                    }
                                } else {
                                    state.directional_light = true;
                                    for light in (&mut lights).join() {
                                        if let Light::Directional(ref mut d) = *light {
                                            d.color = [0.2; 4].into();
                                        }
                                    }
                                }
                            }
                            Some(VirtualKeyCode::P) => if state.point_light {
                                state.point_light = false;
                                state.light_color = [0.0; 4].into();
                            } else {
                                state.point_light = true;
                                state.light_color = [1.0; 4].into();
                            },
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        Trans::None
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

#[derive(Clone)]
struct Assets {
    cube: MeshHandle,
    cone: MeshHandle,
    lid: MeshHandle,
    rectangle: MeshHandle,
    teapot: MeshHandle,
    red: Material,
    white: Material,
    logo: Material,
    font: FontHandle,
}

fn load_assets(world: &World) -> (ProgressCounter, Assets) {
    let mut progress = ProgressCounter::default();
    let mesh_storage = world.read_resource();
    let tex_storage = world.read_resource();
    let font_storage = world.read_resource();
    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let red = loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), &mut progress, &tex_storage);
    let red = Material {
        albedo: red,
        ..mat_defaults.0.clone()
    };

    let white = loader.load_from_data([1.0, 1.0, 1.0, 1.0].into(), &mut progress, &tex_storage);
    let white = Material {
        albedo: white,
        ..mat_defaults.0.clone()
    };

    let logo = Material {
        albedo: loader.load(
            "texture/logo.png",
            PngFormat,
            Default::default(),
            &mut progress,
            &tex_storage,
        ),
        ..mat_defaults.0.clone()
    };

    let cube = loader.load("mesh/cube.obj", ObjFormat, (), &mut progress, &mesh_storage);
    let cone = loader.load("mesh/cone.obj", ObjFormat, (), &mut progress, &mesh_storage);
    let lid = loader.load("mesh/lid.obj", ObjFormat, (), &mut progress, &mesh_storage);
    let teapot = loader.load(
        "mesh/teapot.obj",
        ObjFormat,
        (),
        &mut progress,
        &mesh_storage,
    );
    let rectangle = loader.load(
        "mesh/rectangle.obj",
        ObjFormat,
        (),
        &mut progress,
        &mesh_storage,
    );
    let font = loader.load(
        "font/square.ttf",
        TtfFormat,
        (),
        &mut progress,
        &font_storage,
    );

    (
        progress,
        Assets {
            cube,
            cone,
            lid,
            rectangle,
            teapot,
            red,
            white,
            logo,
            font,
        },
    )
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
        ::std::process::exit(1);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    // Add our meshes directory to the asset loader.
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/renderable/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(DrawUi::new()),
    );
    let game_data = GameDataBuilder::default()
        .with::<ExampleSystem>(ExampleSystem, "example_system", &[])
        .with_bundle(TransformBundle::new().with_dep(&["example_system"]))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(HotReloadBundle::default())?
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?
        .with_bundle(InputBundle::<String, String>::new())?;
    let mut game = Application::build(resources_directory, Loading::default())?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
        .build(game_data)?;
    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    let mut local = Transform::default();
    local.translation = Vector3::new(0., -20., 10.);
    local.rotation = Quaternion::from_angle_x(Deg(75.)).into();
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(local)
        .with(GlobalTransform::default())
        .build();
}
