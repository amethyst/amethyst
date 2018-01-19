//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;

extern crate error_chain;

use error_chain::ChainedError;

use amethyst::{assets, config, core, ecs, renderer, utils, winit, Application, Error, State, Trans};

use assets::{HotReloadBundle, Loader};
use config::Config;
use core::EventsPump;
use core::cgmath::{Array, Deg, Euler, Quaternion, Rad, Rotation, Rotation3, Vector3};
use core::frame_limiter::FrameRateLimitStrategy;
use core::timing::Time;
use core::transform::{LocalTransform, Transform, TransformBundle};
use ecs::{Entity, Fetch, FetchMut, Join, ReadStorage, System, World, WriteStorage};

use renderer::camera::{Camera, Projection};
use renderer::formats::{ObjFormat, PngFormat};
use renderer::hal::{Hal, HalBundle, HalConfig, RendererConfig};
use renderer::light::{AmbientLight, Light, PointLight};
use renderer::material::{Material, MaterialDefaults};
use renderer::mesh::MeshHandle;
use renderer::passes::flat::DrawFlat;
use renderer::system::ActiveGraph;
use renderer::vertex::PosNormTex;

use renderer::gfx_hal::command::{ClearColor, ClearDepthStencil};
use renderer::gfx_hal::format::{AsFormat, D32Float};
use renderer::graph::{ColorAttachment, DepthStencilAttachment, Pass};

// use amethyst::ui::{DrawUi, FontHandle, TtfFormat, UiBundle, UiText, UiTransform};
use utils::fps_counter::{FPSCounter, FPSCounterBundle};
use winit::{ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowEvent};

#[cfg(feature = "gfx-metal")]
use renderer::metal::Backend;

#[cfg(feature = "gfx-vulkan")]
use renderer::vulkan::Backend;

struct DemoState {
    light_angle: f32,
    light_color: [f32; 3],
    ambient_light: bool,
    point_light: bool,
    // directional_light: bool,
    camera_angle: f32,
    // fps_display: Entity,
}

struct ExampleSystem;

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (
        WriteStorage<'a, Light>,
        Fetch<'a, Time>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, LocalTransform>,
        FetchMut<'a, DemoState>,
        // WriteStorage<'a, UiText>,
        Fetch<'a, FPSCounter>,
    );

fn run(&mut self, (mut lights, time, camera, mut transforms, mut state, /*mut ui_text, */
fps_counter): Self::SystemData){
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

        for (transform, light) in (&mut transforms, &mut lights).join() {
            match *light {
                Light::Point(ref mut point_light) => {
                    transform.translation.x = light_orbit_radius * state.light_angle.cos();
                    transform.translation.y = light_orbit_radius * state.light_angle.sin();
                    transform.translation.z = light_z;

                    *point_light = state.light_color.into();
                }
            }
        }

        // if let Some(fps_display) = ui_text.get_mut(state.fps_display) {
        //     if time.frame_number() % 20 == 0 {
        //         let fps = fps_counter.sampled_fps();
        //         fps_display.text = format!("FPS: {:.*}", 2, fps);
        //     }
        // }
    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World) {
        initialise_camera(world);

        let assets = load_assets(&world);

        // Add teapot and lid to scene
        for mesh in vec![assets.lid.clone(), assets.teapot.clone()] {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = Vector3::new(5.0, 5.0, 0.0);

            world
                .create_entity()
                .with(mesh)
                .with(assets.red.clone())
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add cube to scene
        let mut trans = LocalTransform::default();
        trans.translation = Vector3::new(5.0, -5.0, 2.0);
        trans.scale = [2.0; 3].into();

        world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.logo.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cone to scene
        let mut trans = LocalTransform::default();
        trans.translation = Vector3::new(-5.0, 5.0, 0.0);
        trans.scale = [2.0; 3].into();

        world
            .create_entity()
            .with(assets.cone.clone())
            .with(assets.white.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        // Add custom cube object to scene
        let mut trans = LocalTransform::default();
        trans.translation = Vector3::new(-5.0, -5.0, 1.0);
        world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.red.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        // Create base rectangle as floor
        let mut trans = LocalTransform::default();
        trans.scale = Vector3::from_value(10.);

        world
            .create_entity()
            .with(assets.rectangle.clone())
            .with(assets.white.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        // Add lights to scene
        world
            .create_entity()
            .with(Light::from(PointLight([1.0, 1.0, 0.0])))
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        // let light: Light = DirectionalLight {
        //     color: [0.2; 4].into(),
        //     direction: [-1.0; 3],
        // }.into();

        // world.create_entity().with(light).build();

        {
            world.add_resource(AmbientLight([0.01; 3]));
        }

        // let fps_display = world
        //     .create_entity()
        //     .with(UiTransform::new(
        //         "fps".to_string(),
        //         0.,
        //         0.,
        //         1.,
        //         200.,
        //         50.,
        //         0,
        //     ))
        //     .with(UiText::new(
        //         assets.font.clone(),
        //         "N/A".to_string(),
        //         [1.0, 1.0, 1.0, 1.0],
        //         25.,
        //     ))
        //     .build();

        world.add_resource::<DemoState>(DemoState {
            light_angle: 0.0,
            light_color: [1.0; 3],
            ambient_light: true,
            point_light: true,
            // directional_light: true,
            camera_angle: 0.0,
            // fps_display,
        });
    }

    fn handle_event(&mut self, world: &mut World, event: Event) -> Trans {
        let w = world;
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
                                state.light_color = [0.8, 0.2, 0.2];
                            }
                            Some(VirtualKeyCode::G) => {
                                state.light_color = [0.2, 0.8, 0.2];
                            }
                            Some(VirtualKeyCode::B) => {
                                state.light_color = [0.2, 0.2, 0.8];
                            }
                            Some(VirtualKeyCode::W) => {
                                state.light_color = [1.0, 1.0, 1.0];
                            }
                            Some(VirtualKeyCode::A) => {
                                let mut color = w.write_resource::<AmbientLight>();
                                if state.ambient_light {
                                    state.ambient_light = false;
                                    color.0 = [0.0; 3].into();
                                } else {
                                    state.ambient_light = true;
                                    color.0 = [0.01; 3].into();
                                }
                            }
                            Some(VirtualKeyCode::D) => {
                                // let mut lights = w.write::<Light>();

                                // if state.directional_light {
                                //     state.directional_light = false;
                                //     for light in (&mut lights).join() {
                                //         if let Light::Directional(ref mut d) = *light {
                                //             d.color = [0.0; 4].into();
                                //         }
                                //     }
                                // } else {
                                //     state.directional_light = true;
                                //     for light in (&mut lights).join() {
                                //         if let Light::Directional(ref mut d) = *light {
                                //             d.color = [0.2; 4].into();
                                //         }
                                //     }
                                // }
                            }
                            Some(VirtualKeyCode::P) => if state.point_light {
                                state.point_light = false;
                                state.light_color = [0.0; 3].into();
                            } else {
                                state.point_light = true;
                                state.light_color = [1.0; 3].into();
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
}

struct Assets {
    cube: MeshHandle<Backend>,
    cone: MeshHandle<Backend>,
    lid: MeshHandle<Backend>,
    rectangle: MeshHandle<Backend>,
    teapot: MeshHandle<Backend>,
    red: Material<Backend>,
    white: Material<Backend>,
    logo: Material<Backend>,
    // font: FontHandle,
}

fn load_assets(world: &World) -> Assets {
    let mesh_storage = world.read_resource();
    let tex_storage = world.read_resource();
    let mat_defaults = world.read_resource::<MaterialDefaults<Backend>>();
    // let font_storage = world.read_resource();
    let loader = world.read_resource::<Loader>();

    let red = loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), (), &tex_storage);
    let red = Material {
        albedo: red,
        ..mat_defaults.0.clone()
    };

    let white = loader.load_from_data([1.0, 1.0, 1.0, 1.0].into(), (), &tex_storage);
    let white = Material {
        albedo: white,
        ..mat_defaults.0.clone()
    };

    let logo = Material {
        albedo: loader.load(
            "texture/logo.png",
            PngFormat,
            Default::default(),
            (),
            &tex_storage,
        ),
        ..mat_defaults.0.clone()
    };

    let cube = loader.load("mesh/cube.obj", ObjFormat, (), (), &mesh_storage);
    let cone = loader.load("mesh/cone.obj", ObjFormat, (), (), &mesh_storage);
    let lid = loader.load("mesh/lid.obj", ObjFormat, (), (), &mesh_storage);
    let teapot = loader.load("mesh/teapot.obj", ObjFormat, (), (), &mesh_storage);
    let rectangle = loader.load("mesh/rectangle.obj", ObjFormat, (), (), &mesh_storage);
    // let font = loader.load("font/square.ttf", TtfFormat, (), (), &font_storage);

    Assets {
        cube,
        cone,
        lid,
        rectangle,
        teapot,
        red,
        white,
        logo,
        // font,
    }
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

    let mut game = Application::build(resources_directory, Example)?
        .with(ExampleSystem, "example_system", &[])
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
        .with_bundle(TransformBundle::new().with_dep(&["example_system"]))?
        // .with_bundle(UiBundle::new())?
        .with_bundle(HotReloadBundle::default())?
        .with_bundle(FPSCounterBundle::default())?;

    let events_loop = EventsLoop::new();

    let mut hal = HalConfig {
        adapter: None,
        arena_size: 1024 * 1024 * 16,
        chunk_size: 1024,
        min_chunk_size: 512,
        compute: false,
        renderer: Some(RendererConfig {
            title: "Amethyst Hal Example",
            width: 1024,
            height: 768,
            events: &events_loop,
        }),
    }.build::<Backend>()
        .unwrap();

    let mut graph = {
        <DrawFlat as Pass<Backend>>::register(&mut game.world);
        let ref mut renderer = *hal.renderer.as_mut().unwrap();
        let depth = DepthStencilAttachment::new(D32Float::SELF).clear(ClearDepthStencil(1.0, 0));
        let present = ColorAttachment::new(renderer.format)
            .with_clear(ClearColor::Float([0.15, 0.1, 0.2, 1.0]));
        let mut pass = DrawFlat::build().with_color(0, &present).with_depth(&depth);

        renderer
            .add_graph(&[&pass], &present, &hal.device, &mut hal.allocator)
            .unwrap()
    };

    let mut game = game.with_bundle(hal)?
        .with_thread_local(EventsPump(events_loop))
        .build()?;

    (*game.world.write_resource::<ActiveGraph>()).0 = Some(graph);

    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    let mut local = LocalTransform::default();
    local.translation = Vector3::new(0., -20., 10.);
    local.rotation = Quaternion::from_angle_x(Deg(75.)).into();
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(local)
        .with(Transform::default())
        .build();
}
