use amethyst::renderer::{Material, MeshHandle};

/// References to the image data and the coordinates of the individual sprites.
pub struct Sheet {
    /// The coordinates of the individual sprites on the sprite sheet.
    pub sprite_meshes: Vec<MeshHandle>,
    /// The `Material` that stores the sprite sheet texture.
    pub image: Material,
}

impl Sheet {
    /// Returns a sprite sheet
    pub fn new(sprite_meshes: Vec<MeshHandle>, image: Material) -> Self {
        Sheet {
            sprite_meshes,
            image,
        }
    }
}

#[derive(Debug)]
pub struct Metadata {
    /// Width of each individual sprite on the sprite sheet.
    pub sprite_w: f32,
    /// Height of each individual sprite on the sprite sheet.
    pub sprite_h: f32,
    /// Number of rows in the sprite sheet.
    ///
    /// This is the number of sprites counting down the sheet.
    pub row_count: usize,
    /// Number of columns in the sprite sheet.
    ///
    /// This is the number of sprites counting across the sheet.
    pub column_count: usize,
    /// Whether or not there is a 1 pixel border between sprites.
    pub has_border: bool,
}

impl Metadata {
    /// Returns sprite sheet metadata.
    pub fn new(
        sprite_w: f32,
        sprite_h: f32,
        column_count: usize,
        row_count: usize,
        has_border: bool,
    ) -> Self {
        Metadata {
            sprite_w,
            sprite_h,
            column_count,
            row_count,
            has_border,
        }
    }
}
