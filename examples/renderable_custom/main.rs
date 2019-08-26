//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! This particular example use a custom `RenderGraph`.

use amethyst::{
    assets::{
        Completion, Handle, HotReloadBundle, Prefab, PrefabLoader, PrefabLoaderSystemDesc,
        Processor, ProgressCounter, RonFormat,
    },
    core::{
        math::{UnitQuaternion, Vector3},
        timing::Time,
        transform::{Transform, TransformBundle},
    },
    ecs::{
        prelude::{Entity, Join, Read, ReadStorage, System, Write, WriteStorage},
        ReadExpect, SystemData, World,
    },
    input::{
        get_key, is_close_requested, is_key_down, ElementState, InputBundle, StringBindings,
        VirtualKeyCode,
    },
    prelude::*,
    renderer::{
        light::Light,
        mtl::Material,
        palette::{Srgb, Srgba},
        pass::DrawShadedDesc,
        rendy::mesh::{Normal, Position, TexCoord},
        resources::AmbientColor,
        types::DefaultBackend,
        visibility::VisibilitySortingSystem,
        Camera, Factory, Format, GraphBuilder, GraphCreator, Kind, MeshProcessorSystem,
        RenderGroupDesc, RenderingSystem, SpriteSheet, SubpassBuilder, TextureProcessorSystem,
    },
    ui::{DrawUiDesc, UiBundle, UiCreator, UiFinder, UiGlyphsSystemDesc, UiText},
    utils::{
        application_root_dir,
        fps_counter::{FpsCounter, FpsCounterBundle},
        scene::BasicScenePrefab,
    },
    window::{ScreenDimensions, Window, WindowBundle},
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
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.prefab = Some(data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/renderable.ron", RonFormat, &mut self.progress)
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
    let assets_directory = app_root.join("examples").join("assets");

    let display_config_path = app_root
        .join("examples")
        .join("renderable")
        .join("config")
        .join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .with(ExampleSystem::default(), "example_system", &[])
        .with_bundle(TransformBundle::new().with_dep(&["example_system"]))?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(HotReloadBundle::default())?
        .with_bundle(FpsCounterBundle::default())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        // The below Systems, are used to handle some rendering resources.
        // Most likely these must be always called as last thing.
        .with_system_desc(
            UiGlyphsSystemDesc::<DefaultBackend>::default(),
            "ui_glyph_system",
            &[],
        )
        .with(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        )
        .with(
            VisibilitySortingSystem::new(),
            "visibility_sorting_system",
            &[],
        )
        .with(
            MeshProcessorSystem::<DefaultBackend>::default(),
            "mesh_processor",
            &[],
        )
        .with(
            TextureProcessorSystem::<DefaultBackend>::default(),
            "texture_processor",
            &[],
        )
        .with(Processor::<Material>::new(), "material_processor", &[])
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        // The renderer must be executed on the same thread consecutively, so we initialize it as thread_local
        // which will always execute on the main thread.
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            ExampleGraph::default(),
        ));

    let mut game = Application::build(assets_directory, Loading::default())?.build(game_data)?;
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

// This graph structure is used for creating a proper `RenderGraph` for rendering.
// A renderGraph can be thought of as the stages during a render pass. In our case,
// we are only executing one subpass. This graph
// also needs to be rebuilt whenever the window is resized, so the boilerplate code
// for that operation is also here.
#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

#[allow(clippy::map_clone)]
impl GraphCreator<DefaultBackend> for ExampleGraph {
    // This trait method reports to the renderer if the graph must be rebuilt, usually because
    // the window has been resized. This implementation checks the screen size and returns true
    // if it has changed.
    fn rebuild(&mut self, world: &World) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = world.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.deref().clone());
            return false;
        }
        self.dirty
    }

    // This is the core of a RenderGraph, which is building the actual graph with subpasses and target
    // images.
    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        world: &World,
    ) -> GraphBuilder<DefaultBackend, World> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        // Retrieve a reference to the target window, which is created by the WindowBundle
        let window = <ReadExpect<'_, Window>>::fetch(world);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        // Create a new drawing surface in our window
        let surface = factory.create_surface(&window);
        let surface_format = factory.get_surface_format(&surface);

        // Begin building our RenderGraph
        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            // clear screen to black
            Some(ClearValue::Color([0.34, 0.36, 0.52, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        // Create our first `Subpass`, which contains the DrawShaded and DrawUi render groups.
        // We pass the subpass builder a description of our groups for construction
        let pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawShadedDesc::default().builder())
                .with_group(DrawUiDesc::default().builder()) // Draws UI components
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        // Finally, add the pass to the graph
        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(pass));

        graph_builder
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
