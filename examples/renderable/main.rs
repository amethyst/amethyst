//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

use amethyst::{
    assets::{
        Completion, Handle, Prefab, PrefabLoader, PrefabLoaderSystemDesc, ProgressCounter,
        RonFormat,
    },
    core::{
        math::{UnitQuaternion, Vector3},
        timing::Time,
        transform::{Transform, TransformBundle},
    },
    ecs::{Entity, System},
    input::{get_key, is_close_requested, is_key_down, ElementState, InputBundle, VirtualKeyCode},
    prelude::*,
    renderer::{
        light::Light,
        palette::{Srgb, Srgba},
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::{
            hal::command::ClearColor,
            mesh::{Normal, Position, TexCoord},
        },
        resources::AmbientColor,
        types::DefaultBackend,
        Camera, RenderingBundle,
    },
    ui::{RenderUi, UiBundle, UiCreator, UiFinder, UiText},
    utils::{
        application_root_dir,
        fps_counter::{FpsCounter, FpsCounterBundle},
        scene::BasicScenePrefab,
    },
    Error,
};

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

#[derive(Default)]
struct Loading {
    progress: ProgressCounter,
    prefab: Option<Handle<Prefab<MyPrefabData>>>,
}

struct Example {
    scene: Handle<Prefab<MyPrefabData>>,
}

impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        self.prefab = Some(data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/renderable.ron", RonFormat, &mut self.progress)
        }));

        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/fps.ron", &mut self.progress);
            creator.create("ui/loading.ron", &mut self.progress);
        });
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        match self.progress.complete() {
            Completion::Failed => {
                println!("Failed loading assets: {:?}", self.progress.errors());
                Trans::Quit
            }
            Completion::Complete => {
                println!("Assets loaded, swapping state");
                if let Some(entity) = data
                    .world
                    .exec(|finder: UiFinder<'_>| finder.find("loading"))
                {
                    let _ = data.world.delete_entity(entity);
                }
                Trans::Switch(Box::new(Example {
                    scene: self.prefab.as_ref().unwrap().clone(),
                }))
            }
            Completion::Loading => Trans::None,
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData { world, .. } = data;

        world.push((self.scene.clone(),));
    }

    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        let w = data.world;
        if let StateEvent::Window(event) = &event {
            // Exit if user hits Escape or closes the window
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
            match get_key(&event) {
                Some((VirtualKeyCode::R, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = Srgb::new(0.8, 0.2, 0.2);
                    });
                }
                Some((VirtualKeyCode::G, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = Srgb::new(0.2, 0.8, 0.2);
                    });
                }
                Some((VirtualKeyCode::B, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = Srgb::new(0.2, 0.2, 0.8);
                    });
                }
                Some((VirtualKeyCode::W, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = Srgb::new(1.0, 1.0, 1.0);
                    });
                }
                Some((VirtualKeyCode::A, ElementState::Pressed)) => {
                    w.exec(
                        |(mut state, mut color): (
                            Write<'_, DemoState>,
                            Write<'_, AmbientColor>,
                        )| {
                            if state.ambient_light {
                                state.ambient_light = false;
                                color.0 = Srgba::new(0.0, 0.0, 0.0, 0.0);
                            } else {
                                state.ambient_light = true;
                                color.0 = Srgba::new(0.01, 0.01, 0.01, 1.0);
                            }
                        },
                    );
                }
                Some((VirtualKeyCode::D, ElementState::Pressed)) => {
                    w.exec(
                        |(mut state, mut lights): (
                            Write<'_, DemoState>,
                            WriteStorage<'_, Light>,
                        )| {
                            if state.directional_light {
                                state.directional_light = false;
                                for light in (&mut lights).join() {
                                    if let Light::Directional(ref mut d) = *light {
                                        d.color = Srgb::new(0.0, 0.0, 0.0);
                                    }
                                }
                            } else {
                                state.directional_light = true;
                                for light in (&mut lights).join() {
                                    if let Light::Directional(ref mut d) = *light {
                                        d.color = Srgb::new(0.2, 0.2, 0.2);
                                    }
                                }
                            }
                        },
                    );
                }
                Some((VirtualKeyCode::P, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        if state.point_light {
                            state.point_light = false;
                            state.light_color = Srgb::new(0.0, 0.0, 0.0);
                        } else {
                            state.point_light = true;
                            state.light_color = Srgb::new(1.0, 1.0, 1.0);
                        }
                    });
                }
                _ => (),
            }
        }
        Trans::None
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let assets_dir = app_root.join("assets");

    let display_config_path = app_root.join("config").join("display.ron");

    let mut game_data = DispatcherBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .add_bundle(InputBundle::new())?
        .with(
            ExampleSystem::default(),
            "example_system",
            &["input_system"],
        )
        .add_bundle(TransformBundle::new().with_dep(&["example_system"]))?
        .add_bundle(UiBundle::new())?
        .add_bundle(FpsCounterBundle::default())?
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderShaded3D::default())
                .with_plugin(RenderUi::default()),
        )?;
    let game = Application::build(assets_dir, Loading::default())?.build(game_data)?;
    game.run();
    Ok(())
}

struct DemoState {
    light_angle: f32,
    light_color: Srgb,
    ambient_light: bool,
    point_light: bool,
    directional_light: bool,
    camera_angle: f32,
}

impl Default for DemoState {
    fn default() -> Self {
        DemoState {
            light_angle: 0.0,
            light_color: Srgb::new(1.0, 1.0, 1.0),
            ambient_light: true,
            point_light: true,
            directional_light: true,
            camera_angle: 0.0,
        }
    }
}

#[derive(Default)]
struct ExampleSystem {
    fps_display: Option<Entity>,
}

impl<'a> System for ExampleSystem {
    type SystemData = (
        WriteStorage<'a, Light>,
        Read<'a, Time>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
        Write<'a, DemoState>,
        WriteStorage<'a, UiText>,
        Read<'a, FpsCounter>,
        UiFinder<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut lights, time, camera, mut transforms, mut state, mut ui_text, fps_counter, finder) =
            data;
        let light_angular_velocity = -1.0;
        let light_orbit_radius = 15.0;
        let light_z = 6.0;

        let camera_angular_velocity = 0.1;

        state.light_angle += light_angular_velocity * time.delta_seconds();
        state.camera_angle += camera_angular_velocity * time.delta_seconds();

        let delta_rot: UnitQuaternion<f32> = UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            camera_angular_velocity * time.delta_seconds(),
        );
        for (_, transform) in (&camera, &mut transforms).join() {
            // Append the delta rotation to the current transform.
            *transform.isometry_mut() = delta_rot * transform.isometry();
        }

        for (point_light, transform) in
            (&mut lights, &mut transforms)
                .join()
                .filter_map(|(light, transform)| {
                    if let Light::Point(ref mut point_light) = *light {
                        Some((point_light, transform))
                    } else {
                        None
                    }
                })
        {
            transform.set_translation_xyz(
                light_orbit_radius * state.light_angle.cos(),
                light_orbit_radius * state.light_angle.sin(),
                light_z,
            );

            point_light.color = state.light_color;
        }

        if self.fps_display.is_none() {
            if let Some(fps_entity) = finder.find("fps_text") {
                self.fps_display = Some(fps_entity);
            }
        }
        if let Some(fps_entity) = self.fps_display {
            if let Some(fps_display) = ui_text.get_mut(fps_entity) {
                if time.frame_number() % 20 == 0 {
                    let fps = fps_counter.sampled_fps();
                    fps_display.text = format!("FPS: {:.*}", 2, fps);
                }
            }
        }
    }
}
