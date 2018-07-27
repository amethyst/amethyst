//! Displays a shaded sphere to the user.

extern crate amethyst;
#[macro_use]
extern crate serde;

use amethyst::animation::{
    get_animation_set, AnimationBundle, AnimationCommand, AnimationSet, AnimationSetPrefab,
    DeferStartRelation, EndControl, StepDirection,
};
use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::core::{Transform, TransformBundle};
use amethyst::ecs::prelude::Entity;
use amethyst::input::{get_key, is_close_requested, is_key_down};
use amethyst::prelude::*;
use amethyst::renderer::{DrawShaded, ElementState, Event, PosNormTex, VirtualKeyCode};
use amethyst::utils::scene::BasicScenePrefab;

type MyPrefabData = (
    Option<BasicScenePrefab<Vec<PosNormTex>>>,
    Option<AnimationSetPrefab<AnimationId, Transform>>,
);

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
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
        let prefab_handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/animation.ron", RonFormat, (), ())
        });
        self.sphere = Some(world.create_entity().with(prefab_handle).build());
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        let StateData { world, .. } = data;
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
                ).unwrap()
                    .step(self.current_animation, StepDirection::Backward);
            }

            Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).unwrap()
                    .step(self.current_animation, StepDirection::Forward);
            }

            Some((VirtualKeyCode::F, ElementState::Pressed)) => {
                self.rate = 1.0;
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).unwrap()
                    .set_rate(self.current_animation, self.rate);
            }

            Some((VirtualKeyCode::V, ElementState::Pressed)) => {
                self.rate = 0.0;
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).unwrap()
                    .set_rate(self.current_animation, self.rate);
            }

            Some((VirtualKeyCode::H, ElementState::Pressed)) => {
                self.rate = 0.5;
                get_animation_set::<AnimationId, Transform>(
                    &mut world.write_storage(),
                    self.sphere.unwrap().clone(),
                ).unwrap()
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
        Trans::None
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config_path = format!(
        "{}/examples/animation/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
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
