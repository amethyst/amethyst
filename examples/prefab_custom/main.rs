//! Demonstrates loading a custom prefab using the Amethyst engine.

use std::fmt::Debug;

use amethyst::{
    assets::{
        AssetStorage, Handle, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystem,
        ProgressCounter, RonFormat,
    },
    core::{Named, Parent},
    derive::PrefabData,
    ecs::{
        storage::{DenseVecStorage, VecStorage},
        Component, Entities, Entity, Join, ReadStorage, WriteStorage,
    },
    prelude::*,
    utils::application_root_dir,
    Error,
};
use derivative::Derivative;
use derive_new::new;
use serde::{Deserialize, Serialize};
use specs_derive::Component;

#[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Position(pub f32, pub f32, pub f32);

#[derive(Clone, Copy, Component, Debug, Derivative, Deserialize, Serialize, PrefabData)]
#[derivative(Default)]
#[prefab(Component)]
#[storage(VecStorage)]
pub enum Weapon {
    #[derivative(Default)]
    Axe,
    Sword,
}

#[derive(Debug, Default, Deserialize, Serialize, PrefabData)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CustomPrefabData {
    player: Option<Named>,
    weapon: Option<Weapon>,
    position: Option<Position>,
}

#[derive(new)]
pub struct CustomPrefabState {
    /// Tracks loaded assets.
    #[new(default)]
    pub progress_counter: ProgressCounter,
    /// Handle to the loaded prefab.
    #[new(default)]
    pub prefab_handle: Option<Handle<Prefab<CustomPrefabData>>>,
}

// 1. Store the entity that was created.
// 2. Wait for prefab to load.
// 3. Display what was loaded.
// 4. Display the components of the named and weapon entities.
impl SimpleState for CustomPrefabState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data
            .world
            .exec(|loader: PrefabLoader<'_, CustomPrefabData>| {
                loader.load(
                    "prefab/prefab_custom.ron",
                    RonFormat,
                    (),
                    &mut self.progress_counter,
                )
            });

        // Create two sets of entities from the prefab.
        (0..1).for_each(|_| {
            data.world
                .create_entity()
                .with(prefab_handle.clone())
                .build();
        });

        self.prefab_handle = Some(prefab_handle);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress_counter.is_complete() {
            self.display_loaded_prefab(&data.world);
            self.display_loaded_entities(&mut data.world);
            Trans::Quit
        } else {
            Trans::None
        }
    }
}

impl CustomPrefabState {
    // Displays the contents of the loaded prefab.
    fn display_loaded_prefab(&self, world: &World) {
        let prefab_assets = world.read_resource::<AssetStorage<Prefab<CustomPrefabData>>>();
        if let Some(handle) = self.prefab_handle.as_ref() {
            let prefab = prefab_assets
                .get(handle)
                .expect("Expected prefab to be loaded.");

            println!("Prefab");
            println!("======");
            prefab
                .entities()
                .for_each(|entity| println!("{:?}", entity));
            println!("");
        }
    }

    // Displays the `Component`s of entities in the `World`.
    fn display_loaded_entities(&self, world: &mut World) {
        println!("Entities");
        println!("========");
        println!();
        println!(
            "| {e:24} | {prefab_handle:33} | {parent:24} | {pos:23} | {named:22} | {weapon:6} |",
            e = "Entity",
            prefab_handle = "Handle<Prefab<CustomPrefabData>>>",
            parent = "Parent",
            pos = "Position",
            named = "Player",
            weapon = "Weapon",
        );
        println!(
            "| {c:-^24} | {c:-^33} | {c:-^24} | {c:-^23} | {c:-^22} | {c:-^6} |",
            c = "",
        );
        world.exec(
            |(entities, prefab_handles, parents, positions, nameds, weapons): (
                Entities,
                ReadStorage<Handle<Prefab<CustomPrefabData>>>,
                ReadStorage<Parent>,
                ReadStorage<Position>,
                ReadStorage<Named>,
                ReadStorage<Weapon>,
            )| {
                (
                    &entities,
                    prefab_handles.maybe(),
                    parents.maybe(),
                    positions.maybe(),
                    nameds.maybe(),
                    weapons.maybe(),
                )
                    .join()
                    .for_each(|(e, prefab_handle, parent, pos, named, weapon)| {
                        println!(
                            "| {e:24} | {prefab_handle:33} | {parent:24} | {pos:23} | {named:22} | {weapon:6} |",
                            e = format!("{:?}", e),
                            prefab_handle = Self::display(prefab_handle),
                            parent = Self::display(parent.map(|p| p.entity)),
                            pos = Self::display(pos),
                            named = Self::display(named),
                            weapon = Self::display(weapon)
                        )
                    });
            },
        )
    }

    fn display<T: Debug>(component: Option<T>) -> String {
        if let Some(component) = component {
            format!("{:?}", component)
        } else {
            format!("{:?}", component)
        }
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let resources_directory = app_root.join("examples/assets");

    let game_data =
        GameDataBuilder::default().with(PrefabLoaderSystem::<CustomPrefabData>::default(), "", &[]);

    let mut game = Application::new(resources_directory, CustomPrefabState::new(), game_data)?;
    game.run();
    Ok(())
}
