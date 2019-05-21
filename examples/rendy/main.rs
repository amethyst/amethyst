//! Displays spheres with physically based materials.
//!
use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationControlSet, AnimationSet,
        EndControl, VertexSkinningBundle,
    },
    assets::{
        AssetLoaderSystemData, Completion, PrefabLoader, PrefabLoaderSystem, Processor,
        ProgressCounter, RonFormat,
    },
    controls::{FlyControlBundle, FlyControlTag},
    core::{
        ecs::{
            Component, DenseVecStorage, Entities, Entity, Join, Read, ReadExpect, ReadStorage,
            Resources, System, SystemData, Write, WriteStorage,
        },
        math::{Unit, UnitQuaternion, Vector3},
        Time, Transform, TransformBundle,
    },
    gltf::GltfSceneLoaderSystem,
    input::{is_close_requested, is_key_down, Axis, Bindings, Button, InputBundle, StringBindings},
    prelude::*,
    utils::{application_root_dir, fps_counter::FPSCounterBundle, tag::TagFinder},
    window::{ScreenDimensions, Window, WindowBundle},
};
use amethyst_rendy::{
    camera::{ActiveCamera, Camera, Projection},
    debug_drawing::DebugLines,
    light::{Light, PointLight},
    mtl::{Material, MaterialDefaults},
    palette::{LinSrgba, Srgb, Srgba},
    pass::{
        DrawDebugLinesDesc, DrawFlat2DDesc, DrawFlat2DTransparentDesc, DrawFlatDesc,
        DrawFlatTransparentDesc, DrawPbrDesc, DrawPbrTransparentDesc, DrawShadedDesc,
        DrawShadedTransparentDesc, DrawSkyboxDesc,
    },
    rendy::{
        factory::Factory,
        graph::{
            present::PresentNode,
            render::{RenderGroupDesc, SubpassBuilder},
            GraphBuilder,
        },
        hal::{
            command::{ClearDepthStencil, ClearValue},
            format::Format,
            image,
        },
        mesh::{Normal, Position, Tangent, TexCoord},
        texture::palette::load_from_linear_rgba,
    },
    resources::Tint,
    shape::Shape,
    sprite::{SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibilitySortingSystem,
    system::{GraphCreator, RenderingSystem},
    transparent::Transparent,
    types::{Backend, DefaultBackend, Mesh, Texture},
    visibility::{BoundingSphere, VisibilitySortingSystem},
};
use std::{path::Path, sync::Arc};

use prefab_data::{AnimationMarker, Scene, ScenePrefabData, SpriteAnimationId};

mod prefab_data;

struct Example {
    entity: Option<Entity>,
    initialised: bool,
    progress: Option<ProgressCounter>,
}

impl Example {
    pub fn new() -> Self {
        Self {
            entity: None,
            initialised: false,
            progress: None,
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
                0.1,
                1000.0,
            ));
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
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

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 4.0, 8.0);
        transform.prepend_rotation_x_axis(std::f32::consts::PI * -0.0625);

        let camera = world
            .create_entity()
            .with(Camera::from(Projection::perspective(
                1.3,
                std::f32::consts::FRAC_PI_3,
                0.1,
                1000.0,
            )))
            .with(transform)
            .with(FlyControlTag)
            .build();

        world.add_resource(ActiveCamera { entity: camera });
        world.add_resource(RenderMode::default());
        world.add_resource(DebugLines::new());
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
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
        .join("resources")
        .join("display_config.ron");
    let resources = app_root.join("examples").join("assets");

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

    let game_data = GameDataBuilder::default()
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with(OrbitSystem, "orbit", &[])
        .with(CameraCorrectionSystem::new(), "cam", &[])
        // .with_bundle(TransformBundle::new().with_dep(&["orbit"]))?
        .with_bundle(FPSCounterBundle::default())?
        .with(
            PrefabLoaderSystem::<ScenePrefabData>::default(),
            "scene_loader",
            &[],
        )
        .with(
            GltfSceneLoaderSystem::default(),
            "gltf_loader",
            &["scene_loader"], // This is important so that entity instantiation is performed in a single frame.
        )
        .with(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
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
        .with(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &["fly_movement", "cam", "transform_system"],
        )
        .with(
            VisibilitySortingSystem::new(),
            "visibility_system",
            &["fly_movement", "cam", "transform_system"],
        )
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            ExampleGraph::default(),
        ));

    let mut game = Application::new(&resources, Example::new(), game_data)?;
    game.run();
    Ok(())
}

#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    last_mode: RenderMode,
    surface_format: Option<Format>,
    dirty: bool,
}

impl<B: Backend> GraphCreator<B> for ExampleGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        let new_mode = res.fetch::<RenderMode>();

        if *new_mode != self.last_mode {
            self.last_mode = *new_mode;
            return true;
        }

        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    fn builder(&mut self, factory: &mut Factory<B>, res: &Resources) -> GraphBuilder<B, Resources> {
        self.dirty = false;

        let (window, render_mode) =
            <(ReadExpect<'_, Arc<Window>>, ReadExpect<'_, RenderMode>)>::fetch(res);

        let surface = factory.create_surface(&window);

        // cache surface format to speed things up
        let surface_format = *self
            .surface_format
            .get_or_insert_with(|| factory.get_surface_format(&surface));

        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = image::Kind::D2(
            dbg!(dimensions.width()) as u32,
            dimensions.height() as u32,
            1,
            1,
        );

        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0.34, 0.36, 0.52, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let mut opaque_subpass = SubpassBuilder::new();
        let mut transparent_subpass = SubpassBuilder::new();
        match *render_mode {
            RenderMode::Flat => {
                opaque_subpass.add_group(DrawFlatDesc::skinned().builder());
                transparent_subpass.add_group(DrawFlatTransparentDesc::skinned().builder());
            }
            RenderMode::Shaded => {
                opaque_subpass.add_group(DrawShadedDesc::skinned().builder());
                transparent_subpass.add_group(DrawShadedTransparentDesc::skinned().builder());
            }
            RenderMode::Pbr => {
                opaque_subpass.add_group(DrawPbrDesc::skinned().builder());
                transparent_subpass.add_group(DrawPbrTransparentDesc::skinned().builder());
            }
        };

        let opaque = graph_builder.add_node(
            opaque_subpass
                .with_group(DrawFlat2DDesc::new().builder())
                .with_group(DrawDebugLinesDesc::new().builder())
                .with_group(
                    DrawSkyboxDesc::with_colors(
                        Srgb::new(0.82, 0.51, 0.50),
                        Srgb::new(0.18, 0.11, 0.85),
                    )
                    .builder(),
                )
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let transparent = graph_builder.add_node(
            transparent_subpass
                .with_group(
                    DrawFlat2DTransparentDesc::default()
                        .builder()
                        .with_dependency(opaque),
                )
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let _present = graph_builder.add_node(
            PresentNode::builder(factory, surface, color)
                .with_dependency(opaque)
                .with_dependency(transparent),
        );

        graph_builder
    }
}
