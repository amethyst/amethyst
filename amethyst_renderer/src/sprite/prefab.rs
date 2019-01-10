use crate::{Sprite, SpriteRender, SpriteSheet, SpriteSheetHandle, TextureFormat, TexturePrefab};
use amethyst_assets::{AssetStorage, PrefabData, PrefabError, ProgressCounter};
use amethyst_core::specs::{Entity, Read, Write, WriteStorage};

#[derive(Default, Clone, Debug, Deserialize)]
pub struct SpriteRepeat {
    columns: u32,
    rows: Option<u32>,
    count: Option<u32>,
    cell_size: Option<(u32, u32)>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct SpriteGrid {
    width: u32,
    height: u32,
    repeat: SpriteRepeat,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Sprites {
    Manual(Vec<Sprite>),
    Grid(SpriteGrid),
}

#[derive(Clone, Debug, Deserialize)]
pub enum SpriteSheetPrefab {
    #[serde(skip)]
    Handle(SpriteSheetHandle),
    Sheet {
        texture: TexturePrefab<TextureFormat>,
        sprites: Sprites,
    },
}

impl<'a> PrefabData<'a> for SpriteSheetPrefab {
    type SystemData = (
        <TexturePrefab<TextureFormat> as PrefabData<'a>>::SystemData,
        Read<'a, AssetStorage<SpriteSheet>>,
    );
    type Result = SpriteSheetHandle;

    fn add_to_entity(
        &self,
        _entity: Entity,
        _system_data: &mut Self::SystemData,
        _entities: &[Entity],
    ) -> Result<Self::Result, PrefabError> {
        match self {
            SpriteSheetPrefab::Handle(handle) => Ok(handle.clone()),
            _ => unreachable!(),
        }
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let handle = match self {
            SpriteSheetPrefab::Sheet { texture, sprites } => {
                texture.load_sub_assets(progress, &mut system_data.0)?;
                let texture_handle = match texture {
                    TexturePrefab::Handle(handle) => handle.clone(),
                    _ => unreachable!(),
                };
                let spritesheet = SpriteSheet {
                    texture: texture_handle,
                    sprites: sprites.build_sprite_list(),
                };
                Some(
                    (system_data.0)
                        .0
                        .load_from_data(spritesheet, progress, &system_data.1),
                )
            }
            _ => None,
        };
        match handle {
            Some(handle) => {
                *self = SpriteSheetPrefab::Handle(handle);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpriteRenderPrefab {
    /// Index of the sprite sheet in the prefab
    pub sheet: usize,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}

impl<'a> PrefabData<'a> for SpriteRenderPrefab {
    type SystemData = (
        WriteStorage<'a, SpriteRender>,
        Write<'a, SpriteSheetLoadedSet>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut <Self as PrefabData<'a>>::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        system_data
            .0
            .insert(
                entity,
                SpriteRender {
                    sprite_sheet: (system_data.1).0.get(self.sheet).cloned().unwrap(),
                    sprite_number: self.sprite_number,
                },
            )
            .map(|_| ())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpritePrefab {
    sheets: Option<Vec<SpriteSheetPrefab>>,
    render: Option<SpriteRenderPrefab>,
}

impl<'a> PrefabData<'a> for SpritePrefab {
    type SystemData = (
        <SpriteSheetPrefab as PrefabData<'a>>::SystemData,
        <SpriteRenderPrefab as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        if let Some(render) = &self.render {
            render.add_to_entity(entity, &mut system_data.1, entities)?;
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let mut ret = false;
        if let Some(ref mut sheets) = &mut self.sheets {
            ((system_data.1).1).0.clear();
            for sheet in sheets {
                if sheet.load_sub_assets(progress, &mut system_data.0)? {
                    ret = true;
                }
                let sheet = match sheet {
                    SpriteSheetPrefab::Handle(handle) => handle.clone(),
                    _ => unreachable!(),
                };
                ((system_data.1).1).0.push(sheet);
            }
        }
        Ok(ret)
    }
}

impl Sprites {
    fn build_sprite_list(&self) -> Vec<Sprite> {
        match self {
            Sprites::Manual(s) => s.clone(),
            Sprites::Grid(grid) => grid.build_sprite_list(),
        }
    }
}

impl SpriteRepeat {
    fn rows(&self, sheet_height: u32) -> u32 {
        self.rows.unwrap_or_else(|| {
            self.count
                .map(|c| {
                    if (c % self.columns) == 0 {
                        (c / self.columns)
                    } else {
                        (c / self.columns) + 1
                    }
                })
                .or_else(|| self.cell_size.map(|(_, y)| (sheet_height / y)))
                .unwrap_or(1)
        })
    }

    fn count(&self, rows: u32) -> u32 {
        self.count.unwrap_or_else(|| self.columns * rows)
    }

    fn cell_size(&self, rows: u32, sheet_width: u32, sheet_height: u32) -> (u32, u32) {
        self.cell_size
            .unwrap_or_else(|| ((sheet_width / self.columns), (sheet_height / rows)))
    }
}

impl SpriteGrid {
    fn build_sprite_list(&self) -> Vec<Sprite> {
        let rows = self.repeat.rows(self.height);
        let count = self.repeat.count(rows);
        let cell_size = self.repeat.cell_size(rows, self.width, self.height);
        if (self.repeat.columns * cell_size.0) > self.width {
            warn!("Grid spritesheet contain more columns than can fit in the given width: {} * {} > {}",
                  self.repeat.columns,
                  cell_size.0,
                  self.width);
        }
        if (rows * cell_size.1) > self.height {
            warn!(
                "Grid spritesheet contain more rows than can fit in the given height: {} * {} > {}",
                rows, cell_size.1, self.height
            );
        }
        (0..count)
            .map(|cell| {
                let row = cell / rows;
                let column = cell - (row * self.repeat.columns);
                let x = column * cell_size.0;
                let y = row * cell_size.1;
                Sprite::from_pixel_values(
                    self.width,
                    self.height,
                    cell_size.0,
                    cell_size.1,
                    x,
                    y,
                    [0., 0.],
                )
            })
            .collect()
    }
}

#[derive(Default, Clone, Debug)]
pub struct SpriteSheetLoadedSet(pub Vec<SpriteSheetHandle>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_col_row() {
        let sprites = SpriteGrid {
            width: 400,
            height: 200,
            repeat: SpriteRepeat {
                columns: 4,
                rows: Some(4),
                ..Default::default()
            },
        }
        .build_sprite_list();
        println!("{:?}", sprites);
        assert_eq!(16, sprites.len());
        for sprite in &sprites {
            assert_eq!(50., sprite.height);
            assert_eq!(100., sprite.width);
            assert_eq!([0., 0.], sprite.offsets);
        }
        assert_eq!(0., sprites[0].tex_coords.left);
        assert_eq!(0.25, sprites[0].tex_coords.right);
        assert_eq!(1.0, sprites[0].tex_coords.top);
        assert_eq!(0.75, sprites[0].tex_coords.bottom);

        assert_eq!(0.75, sprites[7].tex_coords.left);
        assert_eq!(1.0, sprites[7].tex_coords.right);
        assert_eq!(0.75, sprites[7].tex_coords.top);
        assert_eq!(0.5, sprites[7].tex_coords.bottom);

        assert_eq!(0.25, sprites[9].tex_coords.left);
        assert_eq!(0.5, sprites[9].tex_coords.right);
        assert_eq!(0.5, sprites[9].tex_coords.top);
        assert_eq!(0.25, sprites[9].tex_coords.bottom);
    }

    #[test]
    fn repeat_cell_size_set() {
        assert_eq!(
            (100, 100),
            SpriteRepeat {
                columns: 4,
                cell_size: Some((100, 100)),
                ..Default::default()
            }
            .cell_size(4, 200, 200)
        );
    }

    #[test]
    fn repeat_cell_size_no_set() {
        assert_eq!(
            (50, 100),
            SpriteRepeat {
                columns: 4,
                ..Default::default()
            }
            .cell_size(4, 200, 400)
        );
    }

    #[test]
    fn repeat_count_count_set() {
        assert_eq!(
            12,
            SpriteRepeat {
                columns: 5,
                count: Some(12),
                ..Default::default()
            }
            .count(2)
        );
    }

    #[test]
    fn repeat_count_no_set() {
        assert_eq!(
            10,
            SpriteRepeat {
                columns: 5,
                ..Default::default()
            }
            .count(2)
        );
    }

    #[test]
    fn repeat_rows_rows_set() {
        assert_eq!(
            5,
            SpriteRepeat {
                columns: 5,
                rows: Some(5),
                ..Default::default()
            }
            .rows(400)
        );
    }

    #[test]
    fn repeat_rows_count_set() {
        assert_eq!(
            3,
            SpriteRepeat {
                columns: 5,
                count: Some(12),
                ..Default::default()
            }
            .rows(400)
        );
        assert_eq!(
            3,
            SpriteRepeat {
                columns: 5,
                count: Some(15),
                ..Default::default()
            }
            .rows(400)
        );
    }

    #[test]
    fn repeat_rows_cell_size_set() {
        assert_eq!(
            2,
            SpriteRepeat {
                columns: 5,
                cell_size: Some((200, 200)),
                ..Default::default()
            }
            .rows(400)
        );
        assert_eq!(
            2,
            SpriteRepeat {
                columns: 5,
                cell_size: Some((150, 150)),
                ..Default::default()
            }
            .rows(400)
        );
        assert_eq!(
            2,
            SpriteRepeat {
                columns: 5,
                cell_size: Some((199, 199)),
                ..Default::default()
            }
            .rows(400)
        );
    }

    #[test]
    fn repeat_rows_cell_no_set() {
        assert_eq!(
            1,
            SpriteRepeat {
                columns: 5,
                ..Default::default()
            }
            .rows(400)
        );
    }
}
