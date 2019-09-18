//! Demonstrates loading a simple prefab using the Amethyst engine.

use std::fmt::Debug;

use amethyst::{
    assets::{
        AssetStorage, Handle, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystemDesc,
        ProgressCounter, RonFormat,
    },
    core::Parent,
    derive::PrefabData,
    ecs::{
        storage::DenseVecStorage, Component, Entities, Entity, Join, ReadStorage, World,
        WriteStorage,
    },
    prelude::*,
    utils::application_root_dir,
    Error,
};
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Position(pub f32, pub f32, pub f32);

#[derive(new)]
pub struct CustomPrefabState {
    /// Tracks loaded assets.
    #[new(default)]
    pub progress_counter: ProgressCounter,
    /// Handle to the loaded prefab.
    #[new(default)]
    pub prefab_handle: Option<Handle<Prefab<Position>>>,
}

impl SimpleState for CustomPrefabState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, Position>| {
            loader.load(
                "prefab/prefab_basic.ron",
                RonFormat,
                &mut self.progress_counter,
            )
        });

        // Create one sets of entities from the prefab.
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
        let prefab_assets = world.read_resource::<AssetStorage<Prefab<Position>>>();
        if let Some(handle) = self.prefab_handle.as_ref() {
            let prefab = prefab_assets
                .get(handle)
                .expect("Expected prefab to be loaded.");

            println!("Prefab");
            println!("======");
            prefab
                .entities()
                .for_each(|entity| println!("{:?}", entity));
            println!();
        }
    }

    // Displays the `Component`s of entities in the `World`.
    fn display_loaded_entities(&self, world: &mut World) {
        println!("Entities");
        println!("========");
        println!();
        println!(
            "| {e:24} | {prefab_handle:24} | {parent:6} | {pos:23} |",
            e = "Entity",
            prefab_handle = "Handle<Prefab<Position>>",
            parent = "Parent",
            pos = "Position",
        );
        println!("| {c:-^24} | {c:-^24} | {c:-^6} | {c:-^23} |", c = "",);
        world.exec(
            |(entities, prefab_handles, parents, positions): (
                Entities,
                ReadStorage<Handle<Prefab<Position>>>,
                ReadStorage<Parent>,
                ReadStorage<Position>,
            )| {
                (
                    &entities,
                    prefab_handles.maybe(),
                    parents.maybe(),
                    positions.maybe(),
                )
                    .join()
                    .for_each(|(e, prefab_handle, parent, pos)| {
                        println!(
                            "| {e:24} | {prefab_handle:24} | {parent:6} | {pos:23} |",
                            e = format!("{:?}", e),
                            prefab_handle = Self::display(prefab_handle),
                            parent = Self::display(parent.map(|p| p.entity)),
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
    let assets_dir = app_root.join("examples/assets");

    let game_data = GameDataBuilder::default().with_system_desc(
        PrefabLoaderSystemDesc::<Position>::default(),
        "",
        &[],
    );

    let mut game = Application::new(assets_dir, CustomPrefabState::new(), game_data)?;
    game.run();
    Ok(())
}
