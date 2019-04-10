//! Displays a shaded sphere to the user.

use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationSet, AnimationSetPrefab,
        DeferStartRelation, EndControl, StepDirection,
    },
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    core::{Transform, TransformBundle},
    ecs::prelude::{Entity, ReadExpect, Resources},
    input::{get_key, is_close_requested, is_key_down},
    prelude::*,
    utils::{application_root_dir, scene::BasicScenePrefab},
    window::{EventsLoopSystem, ScreenDimensions, WindowSystem},
    winit::{ElementState, EventsLoop, VirtualKeyCode, Window},
};

use amethyst_rendy::{
    rendy::{factory::Factory, graph::GraphBuilder, hal::Backend, mesh::PosNormTangTex},
    system::{GraphCreator, RendererSystem},
    types::DefaultBackend,
};
use std::{marker::PhantomData, sync::Arc};

use serde::{Deserialize, Serialize};

type MyPrefabData<B> = (
    Option<BasicScenePrefab<B, Vec<PosNormTangTex>>>,
    Option<AnimationSetPrefab<AnimationId, Transform>>,
);

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Scale,
    Rotate,
    Translate,
}

struct Example<B> {
    pub sphere: Option<Entity>,
    rate: f32,
    current_animation: AnimationId,
    _phantom: PhantomData<B>,
}

impl<B: Backend> Default for Example<B> {
    fn default() -> Self {
        Example {
            sphere: None,
            rate: 1.0,
            current_animation: AnimationId::Translate,
            _phantom: PhantomData,
        }
    }
}

impl<B: Backend> SimpleState for Example<B> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        // Initialise the scene with an object, a light and a camera.
        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData<B>>| {
            loader.load("prefab/animation.ron", RonFormat, (), ())
        });
        self.sphere = Some(world.create_entity().with(prefab_handle).build());
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
            match get_key(&event) {
                Some((VirtualKeyCode::Space, ElementState::Pressed)) => {
                    add_animation(
                        world,
                        self.sphere.unwrap(),
                        self.current_animation,
                        self.rate,
                        None,
                        true,
                    );
                }

                Some((VirtualKeyCode::D, ElementState::Pressed)) => {
                    add_animation(
                        world,
                        self.sphere.unwrap(),
                        AnimationId::Translate,
                        self.rate,
                        None,
                        false,
                    );
                    add_animation(
                        world,
                        self.sphere.unwrap(),
                        AnimationId::Rotate,
                        self.rate,
                        Some((AnimationId::Translate, DeferStartRelation::End)),
                        false,
                    );
                    add_animation(
                        world,
                        self.sphere.unwrap(),
                        AnimationId::Scale,
                        self.rate,
                        Some((AnimationId::Rotate, DeferStartRelation::Start(0.666))),
                        false,
                    );
                }

                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap().clone(),
                    )
                    .unwrap()
                    .step(self.current_animation, StepDirection::Backward);
                }

                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap().clone(),
                    )
                    .unwrap()
                    .step(self.current_animation, StepDirection::Forward);
                }

                Some((VirtualKeyCode::F, ElementState::Pressed)) => {
                    self.rate = 1.0;
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap().clone(),
                    )
                    .unwrap()
                    .set_rate(self.current_animation, self.rate);
                }

                Some((VirtualKeyCode::V, ElementState::Pressed)) => {
                    self.rate = 0.0;
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap().clone(),
                    )
                    .unwrap()
                    .set_rate(self.current_animation, self.rate);
                }

                Some((VirtualKeyCode::H, ElementState::Pressed)) => {
                    self.rate = 0.5;
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap().clone(),
                    )
                    .unwrap()
                    .set_rate(self.current_animation, self.rate);
                }

                Some((VirtualKeyCode::R, ElementState::Pressed)) => {
                    self.current_animation = AnimationId::Rotate;
                }

                Some((VirtualKeyCode::S, ElementState::Pressed)) => {
                    self.current_animation = AnimationId::Scale;
                }

                Some((VirtualKeyCode::T, ElementState::Pressed)) => {
                    self.current_animation = AnimationId::Translate;
                }

                _ => {}
            };
        }
        Trans::None
    }
}

fn run<B: Backend>() -> amethyst::Result<()> {
    amethyst::Logger::from_config(amethyst::LoggerConfig {
        log_file: Some("animation_example.log".into()),
        level_filter: log::LevelFilter::Debug,
        ..Default::default()
    })
    .start();

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/animation/resources/display_config.ron");
    let resources = app_root.join("examples/assets/");

    let event_loop = EventsLoop::new();
    let window_system = WindowSystem::from_config_path(&event_loop, display_config_path);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData<B>>::default(), "", &[])
        .with_bundle(AnimationBundle::<AnimationId, Transform>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(TransformBundle::new().with_dep(&["sampler_interpolation_system"]))?
        .with_thread_local(EventsLoopSystem::new(event_loop))
        .with_thread_local(window_system)
        .with_thread_local(RendererSystem::<B, _>::new(ExampleGraph::new()));
    let state: Example<B> = Default::default();
    let mut game = Application::new(resources, state, game_data)?;
    game.run();

    Ok(())
}

fn main() -> amethyst::Result<()> {
    run::<DefaultBackend>()
}

fn add_animation(
    world: &mut World,
    entity: Entity,
    id: AnimationId,
    rate: f32,
    defer: Option<(AnimationId, DeferStartRelation)>,
    toggle_if_exists: bool,
) {
    let animation = world
        .read_storage::<AnimationSet<AnimationId, Transform>>()
        .get(entity)
        .and_then(|s| s.get(&id))
        .cloned()
        .unwrap();
    let mut sets = world.write_storage();
    let control_set = get_animation_set::<AnimationId, Transform>(&mut sets, entity).unwrap();
    match defer {
        None => {
            if toggle_if_exists && control_set.has_animation(id) {
                control_set.toggle(id);
            } else {
                control_set.add_animation(
                    id,
                    &animation,
                    EndControl::Normal,
                    rate,
                    AnimationCommand::Start,
                );
            }
        }

        Some((defer_id, defer_relation)) => {
            control_set.add_deferred_animation(
                id,
                &animation,
                EndControl::Normal,
                rate,
                AnimationCommand::Start,
                defer_id,
                defer_relation,
            );
        }
    }
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

        use amethyst::shred::SystemData;

        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);
        use amethyst_rendy::{
            pass::DrawPbrDesc,
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
            Some(ClearValue::Color(CLEAR_COLOR.into())),
        );

        let depth = graph_builder.create_image(
            surface.kind(),
            1,
            Format::D16Unorm,
            MemoryUsageValue::Data,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let pass = graph_builder.add_node(
            DrawPbrDesc::default()
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
