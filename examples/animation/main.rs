//! Displays a shaded sphere to the user.

use amethyst::assets::Loader;
use amethyst::{
    animation::*,
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    core::{Transform, TransformBundle},
    ecs::prelude::{Entity, ReadExpect, Resources},
    input::{get_key, is_close_requested, is_key_down},
    prelude::*,
    renderer::{
        pass::DrawPbrDesc,
        rendy::mesh::{Normal, Position, Tangent, TexCoord},
        types::DefaultBackend,
        Factory, Format, GraphBuilder, GraphCreator, Kind, RenderGroupDesc, RenderingSystem,
        SubpassBuilder,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
    window::{ScreenDimensions, WindowBundle},
    winit::{ElementState, VirtualKeyCode, Window},
};
use serde::{Deserialize, Serialize};

type MyPrefabData = (
    Option<BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>>,
    Option<AnimationSetPrefab<AnimationId, Transform>>,
);

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Scale,
    Rotate,
    Translate,
    Test,
}

struct Example {
    pub sphere: Option<Entity>,
    rate: f32,
    current_animation: AnimationId,
}

impl Default for Example {
    fn default() -> Self {
        Example {
            sphere: None,
            rate: 1.0,
            current_animation: AnimationId::Translate,
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        // Initialise the scene with an object, a light and a camera.
        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/animation.ron", RonFormat, ())
        });
        self.sphere = Some(world.create_entity().with(prefab_handle).build());

        let (animation_set, animation) = {
            let loader = world.read_resource::<Loader>();

            let sampler = loader.load_from_data(
                Sampler {
                    input: vec![0., 1.],
                    output: vec![
                        SamplerPrimitive::Vec3([0., 0., 0.]),
                        SamplerPrimitive::Vec3([0., 1., 0.]),
                    ],
                    function: InterpolationFunction::Step,
                },
                (),
                &world.read_resource(),
            );

            let animation = loader.load_from_data(
                Animation::new_single(0, TransformChannel::Translation, sampler),
                (),
                &world.read_resource(),
            );
            let mut animation_set: AnimationSet<AnimationId, Transform> = AnimationSet::new();
            animation_set.insert(AnimationId::Test, animation.clone());
            (animation_set, animation)
        };

        let entity = world.create_entity().with(animation_set).build();
        let mut storage = world.write_storage::<AnimationControlSet<AnimationId, Transform>>();
        let control_set = get_animation_set(&mut storage, entity).unwrap();
        control_set.add_animation(
            AnimationId::Test,
            &animation,
            EndControl::Loop(None),
            1.0,
            AnimationCommand::Start,
        );
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
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .step(self.current_animation, StepDirection::Backward);
                }

                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .step(self.current_animation, StepDirection::Forward);
                }

                Some((VirtualKeyCode::F, ElementState::Pressed)) => {
                    self.rate = 1.0;
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .set_rate(self.current_animation, self.rate);
                }

                Some((VirtualKeyCode::V, ElementState::Pressed)) => {
                    self.rate = 0.0;
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .set_rate(self.current_animation, self.rate);
                }

                Some((VirtualKeyCode::H, ElementState::Pressed)) => {
                    self.rate = 0.5;
                    get_animation_set::<AnimationId, Transform>(
                        &mut world.write_storage(),
                        self.sphere.unwrap(),
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

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(amethyst::LoggerConfig {
        level_filter: log::LevelFilter::Error,
        ..Default::default()
    })
    .start();

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/animation/resources/display_config.ron");
    let resources = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(AnimationBundle::<AnimationId, Transform>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(TransformBundle::new().with_dep(&["sampler_interpolation_system"]))?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            ExampleGraph::default(),
        ));
    let state: Example = Default::default();
    let mut game = Application::new(resources, state, game_data)?;
    game.run();

    Ok(())
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

#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

#[allow(clippy::map_clone)]
impl GraphCreator<DefaultBackend> for ExampleGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        self.dirty
    }

    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        use amethyst::shred::SystemData;

        let window = <ReadExpect<'_, Window>>::fetch(res);

        let surface = factory.create_surface(&window);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            factory.get_surface_format(&surface),
            Some(ClearValue::Color(CLEAR_COLOR.into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D16Unorm,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawPbrDesc::skinned().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let present_builder = PresentNode::builder(factory, surface, color).with_dependency(pass);

        graph_builder.add_node(present_builder);

        graph_builder
    }
}
