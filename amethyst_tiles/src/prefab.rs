//! 2D Sprite specific prefabs.
//use crate::{
//    map::{Map, Tile, TileMap},
//    CoordinateEncoder,
//};
//use amethyst_assets::{Handle, PrefabData, ProgressCounter};
//use amethyst_core::ecs::{Entity, Read, WriteStorage};
//use amethyst_error::Error;
//use amethyst_rendy::sprite::{
//    prefab::{SpriteSheetLoadedSet, SpriteSheetReference},
//    SpriteSheet,
//};
//use serde::{Deserialize, Serialize};
//
///// Prefab used to add a sprite to an `Entity`.
/////
///// This prefab is special in that it will lookup the spritesheet in the resource
///// `SpriteSheetLoadedSet` by index during loading. Just like with `SpriteSheetPrefab` this means
///// that this prefab should only be used as part of other prefabs or in specialised formats. Look at
///// `SpriteScenePrefab` for an example.
//#[derive(Clone, Debug, Deserialize, Serialize)]
//#[serde(bound = "")]
//pub struct TileMapPrefab<T: Tile + Serialize + for<'a> Deserialize<'a>, E: CoordinateEncoder> {
//    /// Index of the sprite sheet in the prefab
//    pub sheet: Option<SpriteSheetReference>,
//    /// Index of the sprite on the sprite sheet
//    pub data: TileMap<T, E>,
//
//    #[serde(skip)]
//    handle: Option<Handle<SpriteSheet>>,
//}
//
//impl<'a, T: Tile + Serialize + for<'b> Deserialize<'b>, E: CoordinateEncoder> PrefabData<'a>
//    for TileMapPrefab<T, E>
//{
//    type SystemData = (
//        WriteStorage<'a, TileMap<T, E>>,
//        Read<'a, SpriteSheetLoadedSet>,
//    );
//    type Result = ();
//
//    fn add_to_entity(
//        &self,
//        entity: Entity,
//        system_data: &mut <Self as PrefabData<'a>>::SystemData,
//        _entities: &[Entity],
//        _children: &[Entity],
//    ) -> Result<(), Error> {
//        let mut map = self.data.clone();
//        map.set_sprite_sheet(self.handle.clone());
//        system_data.0.insert(entity, map)?;
//
//        Ok(())
//    }
//
//    fn load_sub_assets(
//        &mut self,
//        _: &mut ProgressCounter,
//        system_data: &mut Self::SystemData,
//    ) -> Result<bool, Error> {
//        if let Some(handle) = (*system_data.1).get(&self.sheet.as_ref().unwrap()) {
//            self.handle = Some(handle);
//            Ok(false)
//        } else {
//            let message = format!("Failed to get `SpriteSheet` with index {:?}.", self.sheet);
//            Err(Error::from_string(message))
//        }
//    }
//}
