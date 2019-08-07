//! Displays spheres with physically based materials.

use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationControlSet, AnimationSet,
        EndControl, VertexSkinningBundle,
    },
    assets::{
        AssetLoaderSystemData, AssetStorage, Completion, Handle, Loader, PrefabLoader,
        PrefabLoaderSystemDesc, ProgressCounter, RonFormat,
    },
    controls::{FlyControlBundle, FlyControlTag},
    core::{
        ecs::{
            Component, DenseVecStorage, DispatcherBuilder, Entities, Entity, Join, Read,
            ReadExpect, ReadStorage, System, SystemData, World, Write, WriteStorage,
        },
        math::{Unit, UnitQuaternion, Vector3},
        SystemDesc, Time, Transform, TransformBundle,
    },
    error::Error,
    gltf::GltfSceneLoaderSystem,
    input::{
        is_close_requested, is_key_down, is_key_up, Axis, Bindings, Button, InputBundle,
        StringBindings,
    },
    prelude::*,
    renderer::{
        bundle::{RenderPlan, RenderPlugin},
        debug_drawing::DebugLines,
        light::{Light, PointLight},
        palette::{LinSrgba, Srgb, Srgba},
        rendy::{
            mesh::{Normal, Position, Tangent, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        resources::Tint,
        shape::Shape,
        types::{DefaultBackend, Mesh, Texture},
        visibility::BoundingSphere,
        ActiveCamera, Camera, Factory, ImageFormat, Material, MaterialDefaults, RenderDebugLines,
        RenderFlat2D, RenderFlat3D, RenderPbr3D, RenderShaded3D, RenderSkybox, RenderToWindow,
        RenderingBundle, SpriteRender, SpriteSheet, SpriteSheetFormat, Transparent,
    },
    utils::{
        application_root_dir,
        auto_fov::{AutoFov, AutoFovSystem},
        fps_counter::FpsCounterBundle,
        tag::TagFinder,
    },
};
use std::path::Path;

use prefab_data::{AnimationMarker, Scene, ScenePrefabData, SpriteAnimationId};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

mod prefab_data;

struct Example {
    entity: Option<Entity>,
    initialised: bool,
    progress: Option<ProgressCounter>,
    bullet_time: bool,
}

impl Example {
    pub fn new() -> Self {
        Self {
            entity: None,
            initialised: false,
            progress: None,
            bullet_time: false,
        }
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
        Write<'a, DebugLines>,
    );

    fn run(&mut self, (time, orbits, mut transforms, mut debug): Self::SystemData) {
        for (orbit, transform) in (&orbits, &mut transforms).join() {
            let angle = time.absolute_time_seconds() as f32 * orbit.time_scale;
            let cross = orbit.axis.cross(&Vector3::z()).normalize() * orbit.radius;
            let rot = UnitQuaternion::from_axis_angle(&orbit.axis, angle);
            let final_pos = (rot * cross) + orbit.center;
            debug.draw_line(
                orbit.center.into(),
                final_pos.into(),
                Srgba::new(0.0, 0.5, 1.0, 1.0),
            );
            transform.set_translation(final_pos);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RenderMode {
    Flat,
    Shaded,
    Pbr,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Pbr
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        #[cfg(feature = "profiler")]
        profile_scope!("example on_start");
        let StateData { world, .. } = data;

        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();

        self.progress = Some(ProgressCounter::default());

        world.exec(
            |(loader, mut scene): (PrefabLoader<'_, ScenePrefabData>, Write<'_, Scene>)| {
                scene.handle = Some(
                    loader.load(
                        Path::new("prefab")
                            .join("rendy_example_scene.ron")
                            .to_string_lossy(),
                        RonFormat,
                        self.progress.as_mut().unwrap(),
                    ),
                );
            },
        );

        let (mesh, albedo) = {
            let mesh = world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
                loader.load_from_data(
                    Shape::Sphere(16, 16)
                        .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(None)
                        .into(),
                    self.progress.as_mut().unwrap(),
                )
            });
            let albedo = world.exec(|loader: AssetLoaderSystemData<'_, Texture>| {
                loader.load_from_data(
                    load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5)).into(),
                    self.progress.as_mut().unwrap(),
                )
            });

            (mesh, albedo)
        };

        println!("Create spheres");
        const NUM_ROWS: usize = 15;
        const NUM_COLS: usize = 15;
        const NUM_PLANES: usize = 2;

        let mut mtls = Vec::with_capacity(100);

        for i in 0..10 {
            for j in 0..10 {
                if mtls.len() >= NUM_ROWS + NUM_COLS - 1 {
                    break;
                }

                let roughness = i as f32 / 9.0;
                let metallic = j as f32 / 9.0;

                let mtl = world.exec(
                    |(mtl_loader, tex_loader): (
                        AssetLoaderSystemData<'_, Material>,
                        AssetLoaderSystemData<'_, Texture>,
                    )| {
                        let metallic_roughness = tex_loader.load_from_data(
                            load_from_linear_rgba(LinSrgba::new(0.0, roughness, metallic, 0.0))
                                .into(),
                            self.progress.as_mut().unwrap(),
                        );

                        mtl_loader.load_from_data(
                            Material {
                                albedo: albedo.clone(),
                                metallic_roughness,
                                ..mat_defaults.clone()
                            },
                            self.progress.as_mut().unwrap(),
                        )
                    },
                );
                mtls.push(mtl);
            }
        }

        for k in 0..NUM_PLANES {
            for i in 0..NUM_COLS {
                for j in 0..NUM_ROWS {
                    let x = i as f32 / (NUM_COLS - 1) as f32;
                    let y = j as f32 / (NUM_ROWS - 1) as f32;
                    let z = k as f32 / (NUM_PLANES - 1) as f32;

                    let center =
                        Vector3::new(15.0 * (x - 0.5), 15.0 * (y - 0.5), 2.0 * (z - 0.5) - 5.0);

                    let mut pos = Transform::default();
                    pos.set_translation(center);
                    pos.set_scale(Vector3::new(0.2, 0.2, 0.2));

                    let mut builder = world
                        .create_entity()
                        .with(pos)
                        .with(mesh.clone())
                        .with(mtls[(j + i) % mtls.len()].clone())
                        .with(Transparent)
                        .with(BoundingSphere::origin(1.0))
                        .with(Orbit {
                            axis: Unit::new_normalize(Vector3::y()),
                            time_scale: 5.0 + y + 0.1 * x + 0.07 * z,
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
        }

        println!("Create lights");
        let light1: Light = PointLight {
            intensity: 6.0,
            color: Srgb::new(0.8, 0.0, 0.0),
            ..PointLight::default()
        }
        .into();

        let mut light1_transform = Transform::default();
        light1_transform.set_translation_xyz(6.0, 6.0, 6.0);

        let light2: Light = PointLight {
            intensity: 5.0,
            color: Srgb::new(0.0, 0.3, 0.7),
            ..PointLight::default()
        }
        .into();

        let mut light2_transform = Transform::default();
        light2_transform.set_translation_xyz(6.0, -6.0, 6.0);

        let light3: Light = PointLight {
            intensity: 4.0,
            color: Srgb::new(0.5, 0.5, 0.5),
            ..PointLight::default()
        }
        .into();

        let mut light3_transform = Transform::default();
        light3_transform.set_translation_xyz(-3.0, 10.0, 2.0);

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

        world
            .create_entity()
            .with(light3)
            .with(light3_transform)
            .build();

        create_tinted_crates(world);

        // Create the camera
        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 4.0, 8.0);

        let mut auto_fov = AutoFov::default();
        auto_fov.set_base_fovx(std::f32::consts::FRAC_PI_3);
        auto_fov.set_base_aspect_ratio(1, 1);

        let camera = world
            .create_entity()
            .with(Camera::standard_3d(16.0, 9.0))
            .with(auto_fov)
            .with(transform)
            .with(FlyControlTag)
            .build();

        world.insert(ActiveCamera {
            entity: Some(camera),
        });
        world.insert(RenderMode::default());
        world.insert(DebugLines::new());
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        #[cfg(feature = "profiler")]
        profile_scope!("example handle_event");
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, winit::VirtualKeyCode::LShift) {
                self.bullet_time = true;
            } else if is_key_up(&event, winit::VirtualKeyCode::LShift) {
                self.bullet_time = false;
            }

            if is_close_requested(&event) || is_key_down(&event, winit::VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, winit::VirtualKeyCode::Space) {
                toggle_or_cycle_animation(
                    self.entity,
                    &mut world.write_resource(),
                    &world.read_storage(),
                    &mut world.write_storage(),
                );
                Trans::None
            } else if is_key_down(&event, winit::VirtualKeyCode::E) {
                let mut mode = world.write_resource::<RenderMode>();
                *mode = match *mode {
                    RenderMode::Flat => RenderMode::Shaded,
                    RenderMode::Shaded => RenderMode::Pbr,
                    RenderMode::Pbr => RenderMode::Flat,
                };
                Trans::None
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        #[cfg(feature = "profiler")]
        profile_scope!("example update");

        {
            let mut time = data.world.write_resource::<Time>();
            time.set_time_scale(if self.bullet_time { 0.2 } else { 1.0 });
        }

        if !self.initialised {
            let remove = match self.progress.as_ref().map(|p| p.complete()) {
                None | Some(Completion::Loading) => false,

                Some(Completion::Complete) => {
                    let scene_handle = data
                        .world
                        .read_resource::<Scene>()
                        .handle
                        .as_ref()
                        .unwrap()
                        .clone();
                    println!("Loading complete.");
                    data.world.create_entity().with(scene_handle).build();
                    true
                }

                Some(Completion::Failed) => {
                    println!("Error: {:?}", self.progress.as_ref().unwrap().errors());
                    return Trans::Quit;
                }
            };
            if remove {
                self.progress = None;
            }
            if self.entity.is_none() {
                if let Some(entity) = data
                    .world
                    .exec(|finder: TagFinder<'_, AnimationMarker>| finder.find())
                {
                    self.entity = Some(entity);
                    self.initialised = true;
                }
            }

            data.world.exec(
                |(entities, animation_sets, mut control_sets): (
                    Entities,
                    ReadStorage<AnimationSet<SpriteAnimationId, SpriteRender>>,
                    WriteStorage<AnimationControlSet<SpriteAnimationId, SpriteRender>>,
                )| {
                    // For each entity that has AnimationSet
                    for (entity, animation_set, _) in (&entities, &animation_sets, !&control_sets)
                        .join()
                        .collect::<Vec<_>>()
                    {
                        // Creates a new AnimationControlSet for the entity
                        let control_set = get_animation_set(&mut control_sets, entity).unwrap();
                        // Adds the `Fly` animation to AnimationControlSet and loops infinitely
                        control_set.add_animation(
                            SpriteAnimationId::Fly,
                            &animation_set.get(&SpriteAnimationId::Fly).unwrap(),
                            EndControl::Loop(None),
                            1.0,
                            AnimationCommand::Start,
                        );
                    }
                },
            );
        }
        Trans::None
    }
}

fn load_crate_spritesheet(world: &mut World) -> Handle<SpriteSheet> {
    let crate_texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            Path::new("texture").join("crate.png").to_string_lossy(),
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let resource_loader = world.read_resource::<Loader>();
    let crate_spritesheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

    resource_loader.load(
        Path::new("texture")
            .join("crate_spritesheet.ron")
            .to_string_lossy(),
        SpriteSheetFormat(crate_texture_handle),
        (),
        &crate_spritesheet_store,
    )
}

fn create_tinted_crates(world: &mut World) {
    let crate_spritesheet = load_crate_spritesheet(world);

    let crate_sprite_render = SpriteRender {
        sprite_sheet: crate_spritesheet.clone(),
        sprite_number: 0,
    };

    let crate_scale = Vector3::new(0.01, 0.01, 1.0);

    let mut red_crate_transform = Transform::default();
    red_crate_transform.set_translation_xyz(4.44, 0.32, 0.5);
    red_crate_transform.set_scale(crate_scale);

    let mut green_crate_transform = Transform::default();
    green_crate_transform.set_translation_xyz(4.44, 0.0, 0.5);
    green_crate_transform.set_scale(crate_scale);

    let mut blue_crate_transform = Transform::default();
    blue_crate_transform.set_translation_xyz(4.44, -0.32, 0.5);
    blue_crate_transform.set_scale(crate_scale);

    world
        .create_entity()
        .with(crate_sprite_render.clone())
        .with(red_crate_transform)
        .with(Tint(Srgb::new(1.0, 0.0, 0.0).into()))
        .build();

    world
        .create_entity()
        .with(crate_sprite_render.clone())
        .with(green_crate_transform)
        .with(Tint(Srgb::new(0.0, 1.0, 0.0).into()))
        .build();

    world
        .create_entity()
        .with(crate_sprite_render.clone())
        .with(blue_crate_transform)
        .with(Tint(Srgb::new(0.0, 0.0, 1.0).into()))
        .build();
}

fn toggle_or_cycle_animation(
    entity: Option<Entity>,
    scene: &mut Scene,
    sets: &ReadStorage<'_, AnimationSet<usize, Transform>>,
    controls: &mut WriteStorage<'_, AnimationControlSet<usize, Transform>>,
) {
    if let Some((entity, Some(animations))) = entity.map(|entity| (entity, sets.get(entity))) {
        if animations.animations.len() > scene.animation_index {
            let animation = animations.animations.get(&scene.animation_index).unwrap();
            let set = get_animation_set::<usize, Transform>(controls, entity).unwrap();
            if set.has_animation(scene.animation_index) {
                set.toggle(scene.animation_index);
            } else {
                println!("Running animation {}", scene.animation_index);
                set.add_animation(
                    scene.animation_index,
                    animation,
                    EndControl::Normal,
                    1.0,
                    AnimationCommand::Start,
                );
            }
            scene.animation_index += 1;
            if scene.animation_index >= animations.animations.len() {
                scene.animation_index = 0;
            }
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(amethyst::LoggerConfig {
        stdout: amethyst::StdoutLog::Off,
        log_file: Some("rendy_example.log".into()),
        level_filter: log::LevelFilter::Error,
        ..Default::default()
    })
    // .level_for("amethyst_utils::fps_counter", log::LevelFilter::Debug)
    // .level_for("rendy_memory", log::LevelFilter::Trace)
    // .level_for("rendy_factory", log::LevelFilter::Trace)
    // .level_for("rendy_resource", log::LevelFilter::Trace)
    // .level_for("rendy_graph", log::LevelFilter::Trace)
    // .level_for("rendy_node", log::LevelFilter::Trace)
    // .level_for("amethyst_rendy", log::LevelFilter::Trace)
    // .level_for("gfx_backend_metal", log::LevelFilter::Trace)
    .start();

    let app_root = application_root_dir()?;

    let display_config_path = app_root
        .join("examples")
        .join("rendy")
        .join("config")
        .join("display.ron");
    let assets_dir = app_root.join("examples").join("assets");

    let mut bindings = Bindings::new();
    bindings.insert_axis(
        "vertical",
        Axis::Emulated {
            pos: Button::Key(winit::VirtualKeyCode::S),
            neg: Button::Key(winit::VirtualKeyCode::W),
        },
    )?;
    bindings.insert_axis(
        "horizontal",
        Axis::Emulated {
            pos: Button::Key(winit::VirtualKeyCode::D),
            neg: Button::Key(winit::VirtualKeyCode::A),
        },
    )?;
    bindings.insert_axis(
        "horizontal",
        Axis::Emulated {
            pos: Button::Key(winit::VirtualKeyCode::D),
            neg: Button::Key(winit::VirtualKeyCode::A),
        },
    )?;

    let game_data = GameDataBuilder::default()
        .with(OrbitSystem, "orbit", &[])
        .with(AutoFovSystem::default(), "auto_fov", &[])
        .with_bundle(FpsCounterBundle::default())?
        .with(
            PrefabLoaderSystemDesc::<ScenePrefabData>::default().build(&mut world),
            "scene_loader",
            &[],
        )
        .with(
            GltfSceneLoaderSystem::new(&mut world),
            "gltf_loader",
            &["scene_loader"], // This is important so that entity instantiation is performed in a single frame.
        )
        .with_bundle(
            AnimationBundle::<usize, Transform>::new("animation_control", "sampler_interpolation")
                .with_dep(&["gltf_loader"]),
        )?
        .with_bundle(
            AnimationBundle::<SpriteAnimationId, SpriteRender>::new(
                "sprite_animation_control",
                "sprite_sampler_interpolation",
            )
            .with_dep(&["gltf_loader"]),
        )?
        .with_bundle(InputBundle::<StringBindings>::new().with_bindings(bindings))?
        .with_bundle(
            FlyControlBundle::<StringBindings>::new(
                Some("horizontal".into()),
                None,
                Some("vertical".into()),
            )
            .with_sensitivity(0.1, 0.1)
            .with_speed(5.),
        )?
        .with_bundle(TransformBundle::new().with_dep(&[
            "animation_control",
            "sampler_interpolation",
            "sprite_animation_control",
            "sprite_sampler_interpolation",
            "fly_movement",
            "orbit",
        ]))?
        .with_bundle(VertexSkinningBundle::new().with_dep(&[
            "transform_system",
            "animation_control",
            "sampler_interpolation",
        ]))?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path))
                .with_plugin(RenderSwitchable3D::default())
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderSkybox::with_colors(
                    Srgb::new(0.82, 0.51, 0.50),
                    Srgb::new(0.18, 0.11, 0.85),
                )),
        )?;

    let mut game = Application::new(
        assets_dir & assets_directory,
        Example::new(),
        game_data,
        world,
    )?;
    game.run();
    Ok(())
}

#[derive(Default, Debug)]
struct RenderSwitchable3D {
    pbr: RenderPbr3D,
    shaded: RenderShaded3D,
    flat: RenderFlat3D,
    last_mode: RenderMode,
}

impl RenderPlugin<DefaultBackend> for RenderSwitchable3D {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        <RenderPbr3D as RenderPlugin<DefaultBackend>>::on_build(&mut self.pbr, world, builder)
    }

    fn should_rebuild(&mut self, world: &World) -> bool {
        let mode = *<Read<'_, RenderMode>>::fetch(world);
        self.last_mode != mode
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<DefaultBackend>,
        factory: &mut Factory<DefaultBackend>,
        world: &World,
    ) -> Result<(), Error> {
        let mode = *<Read<'_, RenderMode>>::fetch(world);
        self.last_mode = mode;
        match mode {
            RenderMode::Pbr => self.pbr.on_plan(plan, factory, world),
            RenderMode::Shaded => self.shaded.on_plan(plan, factory, world),
            RenderMode::Flat => self.flat.on_plan(plan, factory, world),
        }
    }
}
