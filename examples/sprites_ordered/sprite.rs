/// Information about how sprites are laid out on the sprite sheet.
///
/// These are used to calculate the texture coordinates of each sprite.
#[derive(Debug)]
pub struct SpriteSheetDefinition {
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

impl SpriteSheetDefinition {
    /// Returns a new sprite sheet definition.
    ///
    /// # Parameters:
    ///
    /// * `sprite_w`: Width of each individual sprite on the sprite sheet.
    /// * `sprite_h`: Height of each individual sprite on the sprite sheet.
    /// * `row_count`: Number of rows in the sprite sheet.
    /// * `column_count`: Number of columns in the sprite sheet.
    /// * `has_border`: Whether or not there is a 1 pixel border between sprites.
    pub fn new(
        sprite_w: f32,
        sprite_h: f32,
        row_count: usize,
        column_count: usize,
        has_border: bool,
    ) -> Self {
        SpriteSheetDefinition {
            sprite_w,
            sprite_h,
            row_count,
            column_count,
            has_border,
        }
    }
}
