use crate::{
    formats::texture::TexturePrefab,
    sprite::{SpriteRender, SpriteSheet, SpriteSheetHandle, Sprites},
};
use amethyst_assets::{AssetStorage, Loader, PrefabData, ProgressCounter};
use amethyst_core::{
    ecs::{Entity, Read, ReadExpect, WriteStorage},
    Transform,
};
use amethyst_error::Error;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Defines a spritesheet prefab. Note that this prefab will only load the spritesheet in storage,
/// no components will be added to entities. The `add_to_entity` will return the
/// `Handle<SpriteSheet>`. For this reason it is recommended that this prefab is only used as part
/// of other `PrefabData` or in specialised formats. See `SpriteScenePrefab` for an example of this.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(bound = "")]
pub enum SpriteSheetPrefab {
    /// Spritesheet handle, normally not used outside the prefab system.
    #[serde(skip)]
    Handle((Option<String>, SpriteSheetHandle)),
    /// Definition of a spritesheet
    Sheet {
        /// This texture contains the images for the spritesheet
        texture: TexturePrefab,
        /// The sprites in the spritesheet
        sprites: Vec<Sprites>,
        /// The name of the spritesheet to refer to it
        name: Option<String>,
    },
}

impl<'a> PrefabData<'a> for SpriteSheetPrefab {
    type SystemData = (
        <TexturePrefab as PrefabData<'a>>::SystemData,
        Read<'a, SpriteSheetLoadedSet>,
        Read<'a, AssetStorage<SpriteSheet>>,
        ReadExpect<'a, Loader>,
    );
    type Result = (Option<String>, SpriteSheetHandle);

    fn add_to_entity(
        &self,
        _entity: Entity,
        _system_data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<Self::Result, Error> {
        match self {
            SpriteSheetPrefab::Handle(handle) => Ok(handle.clone()),
            _ => unreachable!(),
        }
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (ref mut tex_data, ref mut loaded_set, storage, loader) = system_data;

        if let SpriteSheetPrefab::Sheet {
            texture,
            sprites,
            name,
        } = self
        {
            texture.load_sub_assets(progress, tex_data)?;
            let texture_handle = match texture {
                TexturePrefab::Handle(handle) => handle.clone(),
                _ => unreachable!(),
            };
            let sprites = sprites.iter().flat_map(Sprites::build_sprites).collect();
            let spritesheet = SpriteSheet {
                texture: texture_handle,
                sprites,
            };

            let handle = loader.load_from_data(spritesheet, progress, &storage);
            loaded_set.push((name.clone(), handle.clone()));
            *self = SpriteSheetPrefab::Handle((name.take(), handle));
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
pub struct SpriteSheetLoadedSet(Mutex<Vec<(Option<String>, SpriteSheetHandle)>>);

impl SpriteSheetLoadedSet {
    fn push(&self, data: (Option<String>, SpriteSheetHandle)) {
        self.0.lock().unwrap().push(data);
    }
    fn get(&self, reference: &SpriteSheetReference) -> Option<SpriteSheetHandle> {
        let inner = self.0.lock().unwrap();
        match reference {
            SpriteSheetReference::Index(index) => {
                inner.get(*index).map(|(_, handle)| handle.clone())
            }
            SpriteSheetReference::Name(name) => inner
                .iter()
                .find(|s| s.0.as_ref() == Some(name))
                .map(|(_, handle)| handle.clone()),
        }
    }
}
impl Default for SpriteSheetLoadedSet {
    fn default() -> Self {
        Self(Mutex::new(vec![]))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SpriteSheetReference {
    Index(usize),
    Name(String),
}

/// Prefab used to add a sprite to an `Entity`.
///
/// This prefab is special in that it will lookup the spritesheet in the resource
/// `SpriteSheetLoadedSet` by index during loading. Just like with `SpriteSheetPrefab` this means
/// that this prefab should only be used as part of other prefabs or in specialised formats. Look at
/// `SpriteScenePrefab` for an example.
#[derive(Derivative, Clone, Debug, Deserialize, Serialize)]
#[derivative(Default(bound = ""))]
#[serde(bound = "")]
pub struct SpriteRenderPrefab {
    /// Index of the sprite sheet in the prefab
    pub sheet: Option<SpriteSheetReference>,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,

    #[serde(skip_deserializing, skip_serializing)]
    handle: Option<SpriteSheetHandle>,
}

impl<'a> PrefabData<'a> for SpriteRenderPrefab {
    type SystemData = (
        WriteStorage<'a, SpriteRender>,
        Read<'a, SpriteSheetLoadedSet>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut <Self as PrefabData<'a>>::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<(), Error> {
        if let Some(handle) = self.handle.clone() {
            log::trace!("Creating sprite: {:?}", handle);
            system_data.0.insert(
                entity,
                SpriteRender {
                    sprite_sheet: handle,
                    sprite_number: self.sprite_number,
                },
            )?;
            Ok(())
        } else {
            let message = format!(
                "`SpriteSheetHandle` was not initialized before call to `add_to_entity()`. \
                 sheet: {:?}, sprite_number: {}",
                self.sheet, self.sprite_number
            );
            Err(Error::from_string(message))
        }
    }

    fn load_sub_assets(
        &mut self,
        _: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        if let Some(handle) = (*system_data.1).get(&self.sheet.as_ref().unwrap()) {
            self.handle = Some(handle);
            Ok(false)
        } else {
            let message = format!("Failed to get `SpriteSheet` with index {:?}.", self.sheet);
            Err(Error::from_string(message))
        }
    }
}

/// Prefab for loading a full scene with sprites.
#[derive(Derivative, Clone, Debug, Deserialize, Serialize)]
pub struct SpriteScenePrefab {
    /// Sprite sheets
    pub sheet: Option<SpriteSheetPrefab>,
    /// Add `SpriteRender` to the `Entity`
    pub render: Option<SpriteRenderPrefab>,
    /// Add `Transform` to the `Entity`
    pub transform: Option<Transform>,
}

impl<'a> PrefabData<'a> for SpriteScenePrefab {
    type SystemData = (
        <SpriteSheetPrefab as PrefabData<'a>>::SystemData,
        <SpriteRenderPrefab as PrefabData<'a>>::SystemData,
        <Transform as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        if let Some(render) = &self.render {
            render.add_to_entity(entity, &mut system_data.1, entities, children)?;
        }
        if let Some(transform) = &self.transform {
            transform.add_to_entity(entity, &mut system_data.2, entities, children)?;
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let mut ret = false;
        if let Some(ref mut sheet) = &mut self.sheet {
            if sheet.load_sub_assets(progress, &mut system_data.0)? {
                ret = true;
            }
        }
        if let Some(ref mut render) = &mut self.render {
            render.load_sub_assets(progress, &mut system_data.1)?;
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        rendy::texture::palette::load_from_srgb,
        sprite::{SpriteGrid, SpriteList, SpritePosition, SpriteSheet},
        Texture,
    };
    use amethyst_assets::{Handle, Loader};
    use amethyst_core::ecs::{Builder, Read, ReadExpect, World};
    use rayon::ThreadPoolBuilder;
    use std::sync::Arc;

    fn setup_sprite_world() -> World {
        let mut world = World::new();
        world.register::<SpriteRender>();
        let loader = Loader::new(".", Arc::new(ThreadPoolBuilder::new().build().unwrap()));
        let tex_storage = AssetStorage::<Texture>::default();
        let ss_storage = AssetStorage::<SpriteSheet>::default();
        world.add_resource(tex_storage);
        world.add_resource(ss_storage);
        world.add_resource(SpriteSheetLoadedSet::default());
        world.add_resource(loader);
        world
    }

    fn add_sheet(world: &mut World) -> (SpriteSheetReference, Handle<SpriteSheet>) {
        type Data<'a> = (
            ReadExpect<'a, Loader>,
            Read<'a, AssetStorage<SpriteSheet>>,
            Read<'a, SpriteSheetLoadedSet>,
        );
        let texture = add_texture(world);
        world.exec(|data: Data<'_>| {
            let spritesheet = data.0.load_from_data(
                SpriteSheet {
                    texture,
                    sprites: vec![],
                },
                (),
                &data.1,
            );
            let mut storage = (data.2).0.lock().unwrap();
            let index = storage.len();
            storage.push((None, spritesheet.clone()));
            (SpriteSheetReference::Index(index), spritesheet)
        })
    }

    fn add_texture(world: &mut World) -> Handle<Texture> {
        type Data<'a> = (ReadExpect<'a, Loader>, Read<'a, AssetStorage<Texture>>);
        world.exec(|data: Data<'_>| {
            data.0.load_from_data(
                load_from_srgb(palette::Srgb::new(1., 1., 1.)).into(),
                (),
                &data.1,
            )
        })
    }

    #[test]
    fn sprite_sheet_prefab() {
        let mut world = setup_sprite_world();
        let texture = add_texture(&mut world);
        let mut prefab = SpriteSheetPrefab::Sheet {
            sprites: vec![Sprites::List(SpriteList {
                texture_width: 3,
                texture_height: 1,
                sprites: vec![
                    SpritePosition {
                        x: 0,
                        y: 0,
                        width: 1,
                        height: 1,
                        offsets: None,
                        flip_horizontal: false,
                        flip_vertical: false,
                    },
                    SpritePosition {
                        x: 1,
                        y: 0,
                        width: 1,
                        height: 1,
                        offsets: None,
                        flip_horizontal: false,
                        flip_vertical: false,
                    },
                    SpritePosition {
                        x: 2,
                        y: 0,
                        width: 1,
                        height: 1,
                        offsets: None,
                        flip_horizontal: false,
                        flip_vertical: false,
                    },
                ],
            })],
            texture: TexturePrefab::Handle(texture.clone()),
            name: None,
        };
        prefab
            .load_sub_assets(&mut ProgressCounter::default(), &mut world.system_data())
            .unwrap();
        let h = match &prefab {
            SpriteSheetPrefab::Handle(h) => h.clone(),
            _ => {
                assert!(false);
                return;
            }
        };
        let entity = world.create_entity().build();
        let handle = prefab
            .add_to_entity(entity, &mut world.system_data(), &[entity], &[])
            .unwrap();
        assert_eq!(h, handle);
    }

    #[test]
    fn sprite_render_prefab() {
        let mut world = setup_sprite_world();
        let (sheet, handle) = add_sheet(&mut world);
        let entity = world.create_entity().build();
        let mut prefab = SpriteRenderPrefab {
            sheet: Some(sheet),
            sprite_number: 0,
            handle: None,
        };
        prefab
            .load_sub_assets(&mut ProgressCounter::default(), &mut world.system_data())
            .unwrap();
        prefab
            .add_to_entity(entity, &mut world.system_data(), &[entity], &[])
            .unwrap();
        let storage = world.read_storage::<SpriteRender>();
        let render = storage.get(entity);
        assert!(render.is_some());
        let render = render.unwrap();
        assert_eq!(0, render.sprite_number);
        assert_eq!(handle, render.sprite_sheet);
    }

    #[test]
    fn grid_col_row() {
        let sprites = SpriteGrid {
            texture_width: 400,
            texture_height: 200,
            columns: 4,
            rows: Some(4),
            ..Default::default()
        }
        .build_sprites();

        assert_eq!(16, sprites.len());
        for sprite in &sprites {
            assert_eq!(50., sprite.height);
            assert_eq!(100., sprite.width);
            assert_eq!([0., 0.], sprite.offsets);
        }
        assert_eq!(0., sprites[0].tex_coords.left);
        assert_eq!(0.25, sprites[0].tex_coords.right);
        assert_eq!(0.0, sprites[0].tex_coords.top);
        assert_eq!(0.25, sprites[0].tex_coords.bottom);

        assert_eq!(0.75, sprites[7].tex_coords.left);
        assert_eq!(1.0, sprites[7].tex_coords.right);
        assert_eq!(0.25, sprites[7].tex_coords.top);
        assert_eq!(0.5, sprites[7].tex_coords.bottom);

        assert_eq!(0.25, sprites[9].tex_coords.left);
        assert_eq!(0.5, sprites[9].tex_coords.right);
        assert_eq!(0.5, sprites[9].tex_coords.top);
        assert_eq!(0.75, sprites[9].tex_coords.bottom);

        let sprites = SpriteGrid {
            texture_width: 192,
            texture_height: 64,
            columns: 6,
            rows: Some(2),
            ..Default::default()
        }
        .build_sprites();

        assert_eq!(12, sprites.len());
        for sprite in &sprites {
            assert_eq!(32.0, sprite.height);
            assert_eq!(32.0, sprite.width);
            assert_eq!([0.0, 0.0], sprite.offsets);
        }
        assert_eq!(0.0, sprites[0].tex_coords.left);
        assert_eq!(0.16666667, sprites[0].tex_coords.right);
        assert_eq!(0.0, sprites[0].tex_coords.top);
        assert_eq!(0.5, sprites[0].tex_coords.bottom);

        assert_eq!(0.16666667, sprites[7].tex_coords.left);
        assert_eq!(0.33333334, sprites[7].tex_coords.right);
        assert_eq!(0.5, sprites[7].tex_coords.top);
        assert_eq!(1.0, sprites[7].tex_coords.bottom);

        assert_eq!(0.5, sprites[9].tex_coords.left);
        assert_eq!(0.6666667, sprites[9].tex_coords.right);
        assert_eq!(0.5, sprites[9].tex_coords.top);
        assert_eq!(1.0, sprites[9].tex_coords.bottom);
    }

    #[test]
    fn grid_position() {
        let sprites = SpriteGrid {
            texture_width: 192,
            texture_height: 96,
            columns: 5,
            rows: Some(1),
            cell_size: Some((32, 32)),
            position: Some((32, 32)),
            ..Default::default()
        }
        .build_sprites();

        assert_eq!(5, sprites.len());
        for sprite in &sprites {
            assert_eq!(32.0, sprite.height);
            assert_eq!(32.0, sprite.width);
            assert_eq!([0.0, 0.0], sprite.offsets);
        }

        assert_eq!(0.16666667, sprites[0].tex_coords.left);
        assert_eq!(0.33333334, sprites[0].tex_coords.right);
        assert_eq!(0.33333334, sprites[0].tex_coords.top);
        assert_eq!(0.6666667, sprites[0].tex_coords.bottom);

        assert_eq!(0.8333333, sprites[4].tex_coords.left);
        assert_eq!(1.0, sprites[4].tex_coords.right);
        assert_eq!(0.33333334, sprites[4].tex_coords.top);
        assert_eq!(0.6666667, sprites[4].tex_coords.bottom);
    }

    #[test]
    fn repeat_cell_size_set() {
        assert_eq!(
            (100, 100),
            SpriteGrid {
                texture_width: 200,
                texture_height: 200,
                columns: 4,
                cell_size: Some((100, 100)),
                ..Default::default()
            }
            .cell_size()
        );
    }

    #[test]
    fn repeat_cell_size_no_set() {
        assert_eq!(
            (50, 100),
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 4,
                rows: Some(4),
                ..Default::default()
            }
            .cell_size()
        );
    }

    #[test]
    fn repeat_count_count_set() {
        assert_eq!(
            12,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                sprite_count: Some(12),
                ..Default::default()
            }
            .sprite_count()
        );
    }

    #[test]
    fn repeat_count_no_set() {
        assert_eq!(
            10,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                rows: Some(2),
                ..Default::default()
            }
            .sprite_count()
        );
    }

    #[test]
    fn repeat_rows_rows_set() {
        assert_eq!(
            5,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                rows: Some(5),
                ..Default::default()
            }
            .rows()
        );
    }

    #[test]
    fn repeat_rows_count_set() {
        assert_eq!(
            3,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                sprite_count: Some(12),
                ..Default::default()
            }
            .rows()
        );
        assert_eq!(
            3,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                sprite_count: Some(15),
                ..Default::default()
            }
            .rows()
        );
    }

    #[test]
    fn repeat_rows_cell_size_set() {
        assert_eq!(
            2,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                cell_size: Some((200, 200)),
                ..Default::default()
            }
            .rows()
        );
        assert_eq!(
            2,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                cell_size: Some((150, 150)),
                ..Default::default()
            }
            .rows()
        );
        assert_eq!(
            2,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                cell_size: Some((199, 199)),
                ..Default::default()
            }
            .rows()
        );
    }

    #[test]
    fn repeat_rows_cell_no_set() {
        assert_eq!(
            1,
            SpriteGrid {
                texture_width: 200,
                texture_height: 400,
                columns: 5,
                ..Default::default()
            }
            .rows()
        );
    }
}
