//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

use amethyst::{
    assets::{
        Completion, Handle, HotReloadBundle, Prefab, PrefabLoader, PrefabLoaderSystem,
        ProgressCounter, RonFormat,
    },
    core::{
        nalgebra::{UnitQuaternion, Vector3},
        timing::Time,
        transform::{Transform, TransformBundle},
    },
    ecs::prelude::{Entity, Join, Read, ReadStorage, System, Write, WriteStorage},
    input::{get_key, is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::{AmbientColor, Camera, DrawShaded, ElementState, Light, PosNormTex, VirtualKeyCode},
    ui::{UiBundle, UiCreator, UiFinder, UiText},
    utils::{
        application_root_dir,
        fps_counter::{FPSCounter, FPSCounterBundle},
        scene::BasicScenePrefab,
    },
    Error,
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

#[derive(Default)]
struct Loading {
    progress: ProgressCounter,
    prefab: Option<Handle<Prefab<MyPrefabData>>>,
}

struct Example {
    scene: Handle<Prefab<MyPrefabData>>,
}

impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.prefab = Some(data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/renderable.ron", RonFormat, (), &mut self.progress)
        }));

        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/fps.ron", &mut self.progress);
            creator.create("ui/loading.ron", &mut self.progress);
        });
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
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
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.create_entity().with(self.scene.clone()).build();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let w = data.world;
        if let StateEvent::Window(event) = &event {
            // Exit if user hits Escape or closes the window
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
            match get_key(&event) {
                Some((VirtualKeyCode::R, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = [0.8, 0.2, 0.2, 1.0];
                    });
                }
                Some((VirtualKeyCode::G, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = [0.2, 0.8, 0.2, 1.0];
                    });
                }
                Some((VirtualKeyCode::B, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = [0.2, 0.2, 0.8, 1.0];
                    });
                }
                Some((VirtualKeyCode::W, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        state.light_color = [1.0, 1.0, 1.0, 1.0];
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
                                color.0 = [0.0; 3].into();
                            } else {
                                state.ambient_light = true;
                                color.0 = [0.01; 3].into();
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
                        },
                    );
                }
                Some((VirtualKeyCode::P, ElementState::Pressed)) => {
                    w.exec(|mut state: Write<'_, DemoState>| {
                        if state.point_light {
                            state.point_light = false;
                            state.light_color = [0.0; 4].into();
                        } else {
                            state.point_light = true;
                            state.light_color = [1.0; 4].into();
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
    let resources_directory = app_root.join("examples/assets/");

    let display_config_path = app_root.join("examples/renderable/resources/display_config.ron");

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with::<ExampleSystem>(ExampleSystem::default(), "example_system", &[])
        .with_bundle(TransformBundle::new().with_dep(&["example_system"]))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(HotReloadBundle::default())?
        .with_bundle(FPSCounterBundle::default())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?
        .with_bundle(InputBundle::<String, String>::new())?;
    let mut game = Application::build(resources_directory, Loading::default())?.build(game_data)?;
    game.run();
    Ok(())
}

struct DemoState {
    light_angle: f32,
    light_color: [f32; 4],
    ambient_light: bool,
    point_light: bool,
    directional_light: bool,
    camera_angle: f32,
}

impl Default for DemoState {
    fn default() -> Self {
        DemoState {
            light_angle: 0.0,
            light_color: [1.0; 4],
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

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (
        WriteStorage<'a, Light>,
        Read<'a, Time>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
        Write<'a, DemoState>,
        WriteStorage<'a, UiText>,
        Read<'a, FPSCounter>,
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

        let delta_rot = UnitQuaternion::from_axis_angle(
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
            transform.set_xyz(
                light_orbit_radius * state.light_angle.cos(),
                light_orbit_radius * state.light_angle.sin(),
                light_z,
            );

            point_light.color = state.light_color.into();
        }

        if let None = self.fps_display {
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
