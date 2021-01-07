//! Displays a shaded sphere to the user.

use amethyst::{
    animation::*,
    assets::{DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue, ProgressCounter},
    core::transform::{Transform, TransformBundle},
    input::{get_key, is_close_requested, is_key_down, ElementState, VirtualKeyCode},
    prelude::*,
    renderer::{
        light::{Light, PointLight},
        loaders::load_from_linear_rgba,
        palette::{LinSrgba, Srgb},
        plugins::{RenderPbr3D, RenderToWindow},
        rendy::{
            hal::command::ClearColor,
            mesh::{Normal, Position, Tangent, TexCoord},
        },
        shape::Shape,
        types::{DefaultBackend, MeshData, TextureData},
        Camera, Material, MaterialDefaults, Mesh, RenderingBundle, Texture,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
};
use serde::{Deserialize, Serialize};

const CLEAR_COLOR: ClearColor = ClearColor {
    float32: [0.0, 0.0, 0.0, 1.0],
};

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
    pub progress_counter: Option<ProgressCounter>,
}

impl Default for Example {
    fn default() -> Self {
        Example {
            sphere: None,
            rate: 1.0,
            current_animation: AnimationId::Test,
            progress_counter: Some(ProgressCounter::default()),
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -4.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        let (width, height) = {
            let dim = resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
        };

        world.extend(vec![(Camera::standard_3d(width, height), transform)]);

        let loader = resources.get::<DefaultLoader>().unwrap();

        // Add a sphere

        let mesh: Handle<Mesh> = loader.load_from_data::<Mesh, (), MeshData>(
            Shape::Sphere(64, 64)
                .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(None)
                .into(),
            (),
            &resources.get().unwrap(),
        );

        let albedo = loader.load_from_data::<Texture, (), TextureData>(
            load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5)).into(),
            (),
            &resources.get().unwrap(),
        );

        let mtl: Handle<Material> = {
            let mat_defaults = resources.get::<MaterialDefaults>().unwrap().0.clone();

            loader.load_from_data(
                Material {
                    albedo,
                    ..mat_defaults
                },
                (),
                &resources.get().unwrap(),
            )
        };

        // light it up
        let light1: Light = PointLight {
            intensity: 6.0,
            color: Srgb::new(0.8, 0.0, 0.0),
            ..PointLight::default()
        }
        .into();

        let mut light1_transform = Transform::default();
        light1_transform.set_translation_xyz(6.0, 6.0, -6.0);

        let light2: Light = PointLight {
            intensity: 5.0,
            color: Srgb::new(0.0, 0.3, 0.7),
            ..PointLight::default()
        }
        .into();

        let mut light2_transform = Transform::default();
        light2_transform.set_translation_xyz(6.0, -6.0, -6.0);

        world.extend(vec![(light1, light1_transform), (light2, light2_transform)]);

        // make it dance
        let sampler = loader.load_from_data(
            Sampler::<SamplerPrimitive<f32>> {
                input: vec![0., 1., 2.],
                output: vec![
                    SamplerPrimitive::Vec3([0., 0., 0.]),
                    SamplerPrimitive::Vec3([2., 3., 0.]),
                    SamplerPrimitive::Vec3([1., 2., 3.]),
                ],
                function: InterpolationFunction::Linear,
            },
            (),
            &resources
                .get::<ProcessingQueue<Sampler<SamplerPrimitive<f32>>>>()
                .expect("ProcessingQueue for Sampler"),
        );

        let animation = loader.load_from_data(
            Animation::<Transform>::new_single(0, TransformChannel::Translation, sampler),
            (),
            &resources.get().unwrap(),
        );
        let mut animation_set: AnimationSet<AnimationId, Transform> = AnimationSet::new();
        animation_set.insert(AnimationId::Test, animation);

        self.sphere = Some(world.push((Transform::default(), mesh, mtl, animation_set)));
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let mut query = <(Entity, Read<AnimationSet<AnimationId, Transform>>)>::query();
        let mut buffer = CommandBuffer::new(data.world);

        if let Some(ref progress_counter) = self.progress_counter {
            // Checks progress
            if progress_counter.is_complete() {
                let (query_world, mut subworld) = data.world.split_for_query(&query);
                for (entity, animation_set) in query.iter(&query_world) {
                    // Creates a new AnimationControlSet for the entity
                    if let Some(control_set) =
                        get_animation_set(&mut subworld, &mut buffer, *entity)
                    {
                        if control_set.is_empty() {
                            // Adds the `Fly` animation to AnimationControlSet and loops infinitely
                            control_set.add_animation(
                                AnimationId::Test,
                                &animation_set.get(&AnimationId::Test).unwrap(),
                                EndControl::Loop(None),
                                1.0,
                                AnimationCommand::Start,
                            );
                            self.progress_counter = None;
                        }
                    }
                }
            }
        }
        buffer.flush(data.world);

        Trans::None
    }

    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        let StateData { world, .. } = data;
        let mut buffer = CommandBuffer::new(world);

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
                    get_animation_set::<AnimationId, Transform, World>(
                        world,
                        &mut buffer,
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .step(self.current_animation, StepDirection::Backward);
                }

                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    get_animation_set::<AnimationId, Transform, World>(
                        world,
                        &mut buffer,
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .step(self.current_animation, StepDirection::Forward);
                }

                Some((VirtualKeyCode::F, ElementState::Pressed)) => {
                    self.rate = 1.0;
                    get_animation_set::<AnimationId, Transform, World>(
                        world,
                        &mut buffer,
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .set_rate(self.current_animation, self.rate);
                }

                Some((VirtualKeyCode::V, ElementState::Pressed)) => {
                    self.rate = 0.0;
                    get_animation_set::<AnimationId, Transform, World>(
                        world,
                        &mut buffer,
                        self.sphere.unwrap(),
                    )
                    .unwrap()
                    .set_rate(self.current_animation, self.rate);
                }

                Some((VirtualKeyCode::H, ElementState::Pressed)) => {
                    self.rate = 0.5;
                    get_animation_set::<AnimationId, Transform, World>(
                        world,
                        &mut buffer,
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
        buffer.flush(world);

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
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(AnimationBundle::<AnimationId, Transform>::default())
        .add_bundle(TransformBundle::default())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(CLEAR_COLOR),
                )
                .with_plugin(RenderPbr3D::default()),
        );
    let state: Example = Default::default();
    let game = Application::build(assets_dir, state)?.build(game_data)?;
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
    let animation = {
        let entry = world.entry_ref(entity).unwrap();

        let set = entry
            .get_component::<AnimationSet<AnimationId, Transform>>()
            .expect("AnimationSet for Entity");

        set.get(&id).cloned()
    };

    if let Some(animation) = animation {
        let mut buffer = CommandBuffer::new(world);
        let control_set =
            get_animation_set::<AnimationId, Transform, World>(world, &mut buffer, entity).unwrap();

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

        buffer.flush(world);
    }
}
