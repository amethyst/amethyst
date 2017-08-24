//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate cgmath;

use amethyst::prelude::*;
use amethyst::config::Config;
use amethyst::ecs::{Fetch, FetchMut, Join, System, WriteStorage};
use amethyst::ecs::systems::TransformSystem;
use amethyst::ecs::components::*;
use amethyst::ecs::resources::AmbientColor;
use amethyst::timing::Time;
use amethyst::renderer::prelude::*;
use amethyst::renderer::Rgba;
use amethyst::renderer::Config as DisplayConfig;

use cgmath::{Deg, Euler, Quaternion};
use std::str;

struct DemoState {
    light_angle: f32,
    light_color: [f32; 4],
    ambient_light: bool,
    point_light: bool,
    directional_light: bool,
    camera_angle: f32,
    pipeline_forward: bool,
}

struct ExampleSystem;

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (WriteStorage<'a, LightComponent>,
     Fetch<'a, Time>,
     FetchMut<'a, Camera>,
     FetchMut<'a, DemoState>);

    fn run(&mut self, (mut lights, time, mut camera, mut state): Self::SystemData) {
        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        state.light_angle -= delta_time;
        state.camera_angle += delta_time / 10.0;

        camera.eye[0] = 20.0 * state.camera_angle.cos();
        camera.eye[1] = 20.0 * state.camera_angle.sin();

        for point_light in (&mut lights).join()
            .filter_map(|light| {
                if let LightComponent(Light::Point(ref mut point_light)) = *light {
                    Some(point_light)
                } else {
                    None
                }
            }) {
            point_light.center[0] = 15.0 * state.light_angle.cos();
            point_light.center[1] = 15.0 * state.light_angle.sin();
            point_light.center[2] = 6.0;

            point_light.color = state.light_color.into();
        }

    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        let world = &mut engine.world;

        world.add_resource(Camera {
            eye: [0.0, -20.0, 10.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 0.0, 1.0].into(),
        });

        world.register::<LocalTransform>();
        world.register::<Child>();
        world.register::<Init>();
        world.register::<LightComponent>();

        // FIXME: asset loader pending
        // Set up an assets path by directly registering an assets store.
        /*let assets_path = format!("{}/examples/03_renderable/resources/meshes",
                                  env!("CARGO_MANIFEST_DIR"));
        assets.register_store(DirectoryStore::new(assets_path));

        // Create some basic colors and load textures
        assets.load_asset_from_data::<Texture, [f32; 4]>("red", [0.8, 0.2, 0.2, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("green", [0.2, 0.8, 0.2, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("blue", [0.2, 0.2, 0.8, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("pink", [1.0, 0.8, 0.8, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("black", [0.0, 0.0, 0.0, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        assets.load_asset::<Texture>("logo", "png");
        assets.load_asset::<Texture>("ground", "dds");

        // Load/generate meshes
        assets.load_asset::<Mesh>("teapot", "obj");
        assets.load_asset::<Mesh>("lid", "obj");
        assets.load_asset::<Mesh>("rectangle", "obj");
        assets.load_asset::<Mesh>("cube", "obj");
        assets.load_asset::<Mesh>("cone", "obj");*/

        // Add teapot and lid to scene
        for mesh in vec!["lid", "teapot"].iter() {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 5.0, 0.0];
            // FIXME: asset loader pending
            /*let rend = assets
                .create_renderable(mesh, "red", "blue", "white", 10.0)
                .unwrap();*/
            world
                .create_entity()
                //.with(rend) // FIXME: asset loader pending
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add cube to scene
        // FIXME: asset loader pending
//        let rend = assets
//            .create_renderable("cube", "logo", "logo", "white", 1.0)
//            .unwrap();
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, -5.0, 2.0];
        trans.scale = [2.0; 3];
        world
            .create_entity()
            // .with(rend) // FIXME: asset loader pending
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cone to scene
        // FIXME: asset loader pending
        /*let rend = assets
            .create_renderable("cone", "white", "red", "blue", 40.0)
            .unwrap();*/
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 5.0, 0.0];
        trans.scale = [2.0; 3];
        world
            .create_entity()
            //.with(rend) // FIXME: asset loader pending
            .with(trans)
            .with(Transform::default())
            .build();

        // Add custom cube object to scene
        // FIXME: asset loader pending
        /*let rend = assets
            .create_renderable("cube", "blue", "green", "white", 1.0)
            .unwrap();*/
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, -5.0, 1.0];
        world
            .create_entity()
            //.with(rend) // FIXME: asset loader pending
            .with(trans)
            .with(Transform::default())
            .build();

        // Create base rectangle as floor
        // FIXME: asset loader pending
        /*let rend = assets
            .create_renderable("rectangle", "ground", "ground", "black", 1.0)
            .unwrap();*/
        let mut trans = LocalTransform::default();
        trans.scale = [10.0; 3];
        world
            .create_entity()
            //.with(rend) // FIXME: asset loader pending
            .with(trans)
            .with(Transform::default())
            .build();

        // Add lights to scene
        world.create_entity()
            .with(LightComponent(PointLight::default().into()))
            .build();

        world
            .create_entity()
            .with(LightComponent(DirectionalLight {
                      color: [0.2; 4].into(),
                      direction: [-1.0; 3].into(),
                  }.into()))
            .build();

        {
            world.add_resource(Some(AmbientColor(Rgba::from([0.01; 3]))));
        }

        world.add_resource::<DemoState>(DemoState {
                                            light_angle: 0.0,
                                            light_color: [1.0; 4],
                                            ambient_light: true,
                                            point_light: true,
                                            directional_light: true,
                                            camera_angle: 0.0,
                                            pipeline_forward: true,
                                        });
    }

    fn handle_event(&mut self, engine: &mut Engine, event: Event) -> Trans {
        let w = &mut engine.world;
        // Exit if user hits Escape or closes the window
        let mut state = w.write_resource::<DemoState>();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Closed => return Trans::Quit,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode,
                        state : ElementState::Pressed,
                        ..
                    }, ..
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
                        },
                        Some(VirtualKeyCode::R) => {
                            state.light_color = [0.8, 0.2, 0.2, 1.0];
                        },
                        Some(VirtualKeyCode::G) => {
                            state.light_color = [0.2, 0.8, 0.2, 1.0];
                        },
                        Some(VirtualKeyCode::B) => {
                            state.light_color = [0.2, 0.2, 0.8, 1.0];
                        },
                        Some(VirtualKeyCode::W) => {
                            state.light_color = [1.0, 1.0, 1.0, 1.0];
                        },
                        Some(VirtualKeyCode::A) => {
                            if let Some(ref mut color) = *w.write_resource::<Option<AmbientColor>>() {
                                if state.ambient_light {
                                    state.ambient_light = false;
                                    color.0 = [0.0; 3].into();
                                } else {
                                    state.ambient_light = true;
                                    color.0 = [0.01; 3].into();
                                }
                            }
                        },
                        Some(VirtualKeyCode::D) => {
                            let mut lights = w.write::<LightComponent>();

                            if state.directional_light {
                                state.directional_light = false;
                                for mut light in (&mut lights).join() {
                                    if let LightComponent(Light::Directional(ref mut d)) = *light {
                                        d.color = [0.0; 4].into();
                                    }
                                }
                            } else {
                                state.directional_light = true;
                                for mut light in (&mut lights).join() {
                                    if let LightComponent(Light::Directional(ref mut d)) = *light {
                                        d.color = [0.2; 4].into();
                                    }
                                }
                            }
                        },
                        Some(VirtualKeyCode::P) => {
                            if state.point_light {
                                state.point_light = false;
                                state.light_color = [0.0; 4].into();
                            } else {
                                state.point_light = true;
                                state.light_color = [1.0; 4].into();
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            },
            _ => (),
        }
        Trans::None
    }
}

fn main() {
    let path = format!("{}/examples/03_renderable/resources/config.ron",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::load(path);
    let mut game = Application::build(Example).unwrap()
        .with::<ExampleSystem>(ExampleSystem, "example_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
        .with_renderer(Pipeline::build()
                           .with_stage(Stage::with_backbuffer()
                               .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                               .with_model_pass(pass::DrawFlat::<PosNormTex>::new())
                           ),
                       Some(cfg)).unwrap()
        .build()
        .expect("Fatal error");
    game.run();
}
