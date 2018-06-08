//! Displays a shaded sphere to the user.

extern crate amethyst;

use amethyst::animation::{get_animation_set, Animation, AnimationBundle, AnimationCommand,
                          AnimationSet, DeferStartRelation, EndControl, InterpolationFunction,
                          Sampler, SamplerPrimitive, StepDirection, TransformChannel};
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::cgmath::Deg;
use amethyst::core::{GlobalTransform, Parent, Transform, TransformBundle};
use amethyst::ecs::prelude::{Entity, World};
use amethyst::input::{get_key, is_close_requested, is_key};
use amethyst::prelude::*;
use amethyst::renderer::{AmbientColor, Camera, DrawShaded, Event, Light, Mesh, PointLight,
                         PosNormTex, Projection, Rgba, VirtualKeyCode, Shape};

// blue
const SPHERE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
// near-black
const AMBIENT_LIGHT_COLOUR: Rgba = Rgba(0.01, 0.01, 0.01, 1.0);
// white
const POINT_LIGHT_COLOUR: Rgba = Rgba(1.0, 1.0, 1.0, 1.0);
const LIGHT_POSITION: [f32; 3] = [2.0, 2.0, -2.0];
const LIGHT_RADIUS: f32 = 5.0;
const LIGHT_INTENSITY: f32 = 3.0;

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone)]
enum AnimationId {
    Scale,
    Rotate,
    Translate,
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

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        // Initialise the scene with an object, a light and a camera.
        let sphere_entity = initialise_sphere(world);
        self.sphere = Some(sphere_entity);
        initialise_animation(world, sphere_entity);
        initialise_lights(world);
        initialise_camera(world);
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        let StateData { world, .. } = data;
        if is_close_requested(&event) || is_key(&event, VirtualKeyCode::Escape) {
            return Trans::Quit;
        }
        match get_key(&event) {
            Some(VirtualKeyCode::Space) => {
                add_animation(
                    world,
                    self.sphere.unwrap(),
                    self.current_animation,
                    self.rate,
                    None,
                    true,
                );
            }

            Some(VirtualKeyCode::D) => {
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

            Some(VirtualKeyCode::Left) => {
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).step(self.current_animation, StepDirection::Backward);
            }

            Some(VirtualKeyCode::Right) => {
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).step(self.current_animation, StepDirection::Forward);
            }

            Some(VirtualKeyCode::F) => {
                self.rate = 1.0;
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).set_rate(self.current_animation, self.rate);
            }

            Some(VirtualKeyCode::V) => {
                self.rate = 0.0;
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).set_rate(self.current_animation, self.rate);
            }

            Some(VirtualKeyCode::H) => {
                self.rate = 0.5;
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).set_rate(self.current_animation, self.rate);
            }

            Some(VirtualKeyCode::R) => {
                self.current_animation = AnimationId::Rotate;
            }

            Some(VirtualKeyCode::S) => {
                self.current_animation = AnimationId::Scale;
            }

            Some(VirtualKeyCode::T) => {
                self.current_animation = AnimationId::Translate;
            }

            _ => {}
        };
        Trans::None
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    let display_config_path = format!(
        "{}/examples/animation/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with_bundle(AnimationBundle::<AnimationId, Transform>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(TransformBundle::new().with_dep(&["sampler_interpolation_system"]))?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;
    let mut game = Application::new(resources, Example::default(), game_data)?;
    game.run();

    Ok(())
}

/// This function initialises a sphere and adds it to the world.
fn initialise_sphere(world: &mut World) -> Entity {
    // Create a sphere mesh and material.

    use amethyst::assets::Handle;
    use amethyst::renderer::{Material, MaterialDefaults};

    let (mesh, material) = {
        let loader = world.read_resource::<Loader>();

        let mesh: Handle<Mesh> = loader.load_from_data(
            Shape::Sphere(32, 32).generate::<Vec<PosNormTex>>(None),
            (),
            &world.read_resource(),
        );

        let albedo = SPHERE_COLOUR.into();

        let tex_storage = world.read_resource();
        let mat_defaults = world.read_resource::<MaterialDefaults>();

        let albedo = loader.load_from_data(albedo, (), &tex_storage);

        let mat = Material {
            albedo,
            ..mat_defaults.0.clone()
        };

        (mesh, mat)
    };

    let parent_entity = world
        .create_entity()
        .with(Transform::default())
        .with(GlobalTransform::default())
        .build();

    // Create a sphere entity using the mesh and the material.
    world
        .create_entity()
        .with(Transform {
            translation: [0., 1.0, 0.].into(),
            ..Transform::default()
        })
        .with(GlobalTransform::default())
        .with(Parent {
            entity: parent_entity.clone(),
        })
        .with(mesh)
        .with(material)
        .build();

    parent_entity
}

fn initialise_animation(world: &mut World, entity: Entity) {
    let loader = world.write_resource::<Loader>();
    let translation_sampler = Sampler::<SamplerPrimitive<f32>> {
        input: vec![0., 1., 2., 3., 4.],
        function: InterpolationFunction::Linear,
        output: vec![
            [0., 0., 0.].into(),
            [1., 0., 0.].into(),
            [0., 0., 0.].into(),
            [-1., 0., 0.].into(),
            [0., 0., 0.].into(),
        ],
    };

    let scale_sampler = Sampler::<SamplerPrimitive<f32>> {
        input: vec![0., 1., 2., 3., 4.],
        function: InterpolationFunction::Linear,
        output: vec![
            [1., 1., 1.].into(),
            [0.6, 0.6, 0.6].into(),
            [0.3, 0.3, 0.3].into(),
            [0.6, 0.6, 0.6].into(),
            [1., 1., 1.].into(),
        ],
    };

    use std::f32::consts::FRAC_1_SQRT_2;
    let rotation_sampler = Sampler::<SamplerPrimitive<f32>> {
        input: vec![0., 1., 2., 3., 4.],
        function: InterpolationFunction::SphericalLinear,
        output: vec![
            [1., 0., 0., 0.].into(),
            [FRAC_1_SQRT_2, 0., 0., FRAC_1_SQRT_2].into(),
            [0., 0., 0., 1.].into(),
            [-FRAC_1_SQRT_2, 0., 0., FRAC_1_SQRT_2].into(),
            [-1., 0., 0., 0.].into(),
        ],
    };
    let translation_sampler_handle =
        loader.load_from_data(translation_sampler, (), &world.read_resource());
    let scale_sampler_handle = loader.load_from_data(scale_sampler, (), &world.read_resource());
    let rotation_sampler_handle =
        loader.load_from_data(rotation_sampler, (), &world.read_resource());
    let animation_storage = world.read_resource();
    let mut set = AnimationSet::<AnimationId, Transform>::new();
    add_to_set(
        &mut set,
        AnimationId::Translate,
        TransformChannel::Translation,
        translation_sampler_handle,
        &loader,
        &animation_storage,
    );
    add_to_set(
        &mut set,
        AnimationId::Scale,
        TransformChannel::Scale,
        scale_sampler_handle,
        &loader,
        &animation_storage,
    );
    add_to_set(
        &mut set,
        AnimationId::Rotate,
        TransformChannel::Rotation,
        rotation_sampler_handle,
        &loader,
        &animation_storage,
    );
    // entity created just before this function is called, so unwrap is safe
    world.write_storage().insert(entity, set).unwrap();
}

fn add_to_set(
    set: &mut AnimationSet<AnimationId, Transform>,
    id: AnimationId,
    channel: TransformChannel,
    sampler: Handle<Sampler<SamplerPrimitive<f32>>>,
    loader: &Loader,
    animation_storage: &AssetStorage<Animation<Transform>>,
) {
    set.insert(
        id,
        loader.load_from_data(
            Animation::new_single(0, channel, sampler),
            (),
            animation_storage,
        ),
    );
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
    let control_set = get_animation_set::<AnimationId, Transform>(&mut sets, entity);
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

/// This function adds an ambient light and a point light to the world.
fn initialise_lights(world: &mut World) {
    // Add ambient light.
    world.add_resource(AmbientColor(AMBIENT_LIGHT_COLOUR));

    let light: Light = PointLight {
        center: LIGHT_POSITION.into(),
        radius: LIGHT_RADIUS,
        intensity: LIGHT_INTENSITY,
        color: POINT_LIGHT_COLOUR,
        ..Default::default()
    }.into();

    // Add point light.
    world.create_entity().with(light).build();
}

/// This function initialises a camera and adds it to the world.
fn initialise_camera(world: &mut World) {
    use amethyst::core::cgmath::Matrix4;
    let transform =
        Matrix4::from_translation([0.0, 0.0, -4.0].into()) * Matrix4::from_angle_y(Deg(180.));
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(GlobalTransform(transform.into()))
        .build();
}
