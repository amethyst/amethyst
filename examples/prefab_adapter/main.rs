//! Demonstrates loading a custom prefab using the Amethyst engine.

use std::fmt::Debug;

use amethyst::{
    assets::{
        AssetStorage, Handle, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystem,
        ProgressCounter, RonFormat,
    },
    ecs::{storage::DenseVecStorage, Component, Entities, Entity, Join, ReadStorage, WriteStorage},
    prelude::*,
    utils::application_root_dir,
    Error,
};
use derive_new::new;
use serde::{Deserialize, Serialize};
use specs_derive::Component;

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct Position(pub f32, pub f32, pub f32);

impl From<(i32, i32, i32)> for Position {
    fn from((x, y, z): (i32, i32, i32)) -> Position {
        Position(x as f32, y as f32, z as f32)
    }
}

impl From<(f32, f32, f32)> for Position {
    fn from((x, y, z): (f32, f32, f32)) -> Position {
        Position(x, y, z)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum PositionPrefab {
    Pos3f { x: f32, y: f32, z: f32 },
    Pos3i { x: i32, y: i32, z: i32 },
}

impl<'a> PrefabData<'a> for PositionPrefab {
    // To attach the `Position` to the constructed entity,
    // we write to the `Position` component storage.
    type SystemData = WriteStorage<'a, Position>;

    // This associated type is not used in this pattern,
    // so the empty tuple is specified.
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        positions: &mut Self::SystemData,
        _entities: &[Entity],
    ) -> Result<(), Error> {
        let position = match *self {
            PositionPrefab::Pos3f { x, y, z } => (x, y, z).into(),
            PositionPrefab::Pos3i { x, y, z } => (x, y, z).into(),
        };
        positions.insert(entity, position).map(|_| ())?;
        Ok(())
    }
}

#[derive(new)]
pub struct CustomPrefabState {
    /// Tracks loaded assets.
    #[new(default)]
    pub progress_counter: ProgressCounter,
    /// Handle to the loaded prefab.
    #[new(default)]
    pub prefab_handle: Option<Handle<Prefab<PositionPrefab>>>,
}

impl SimpleState for CustomPrefabState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, PositionPrefab>| {
            loader.load(
                "prefab/prefab_adapter.ron",
                RonFormat,
                (),
                &mut self.progress_counter,
            )
        });

        // Create one set of entities from the prefab.
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
        let prefab_assets = world.read_resource::<AssetStorage<Prefab<PositionPrefab>>>();
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
            "| {e:24} | {prefab_handle:30} | {pos:23} |",
            e = "Entity",
            prefab_handle = "Handle<Prefab<PositionPrefab>>",
            pos = "Position",
        );
        println!("| {c:-^24} | {c:-^30} | {c:-^23} |", c = "",);
        world.exec(
            |(entities, prefab_handles, positions): (
                Entities,
                ReadStorage<Handle<Prefab<PositionPrefab>>>,
                ReadStorage<Position>,
            )| {
                (&entities, prefab_handles.maybe(), positions.maybe())
                    .join()
                    .for_each(|(e, prefab_handle, pos)| {
                        println!(
                            "| {e:24} | {prefab_handle:30} | {pos:23} |",
                            e = format!("{:?}", e),
                            prefab_handle = Self::display(prefab_handle),
                            pos = Self::display(pos),
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
        GameDataBuilder::default().with(PrefabLoaderSystem::<PositionPrefab>::default(), "", &[]);

    let mut game = Application::new(resources_directory, CustomPrefabState::new(), game_data)?;
    game.run();
    Ok(())
}
