use amethyst_assets::{AssetPrefab, Format, PrefabData, PrefabError, ProgressCounter};
use amethyst_core::specs::prelude::{Entity, WriteStorage};
use amethyst_renderer::{Texture, TextureMetadata, TexturePrefab};

use {Anchor, FontAsset, MouseReactive, Stretch, TextEditing, UiImage, UiText, UiTransform};

/// Loadable `UiTransform` data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct UiTransformBuilder {
    id: String,
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    tab_order: i32,
    opaque: bool,
    percent: bool,
    stretch: Option<Stretch>,
    anchor: Anchor,
    mouse_reactive: bool,
}

impl Default for UiTransformBuilder {
    fn default() -> Self {
        UiTransformBuilder {
            id: "".to_string(),
            x: 0.,
            y: 0.,
            z: 0.,
            width: 0.,
            height: 0.,
            tab_order: 0,
            opaque: true,
            percent: false,
            stretch: None,
            anchor: Anchor::Middle,
            mouse_reactive: false,
        }
    }
}

impl UiTransformBuilder {
    /// Set id
    pub fn with_id<S>(mut self, id: S) -> Self
    where
        S: ToString,
    {
        self.id = id.to_string();
        self
    }

    /// Set position
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// Set size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set tab order
    pub fn with_tab_order(mut self, tab_order: i32) -> Self {
        self.tab_order = tab_order;
        self
    }

    /// Set to event transparent
    pub fn transparent(mut self) -> Self {
        self.opaque = false;
        self
    }

    /// Add mouse reactive
    pub fn reactive(mut self) -> Self {
        self.mouse_reactive = true;
        self
    }

    /// Set anchor
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set stretch
    pub fn with_stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = Some(stretch);
        self
    }
}

impl<'a> PrefabData<'a> for UiTransformBuilder {
    type SystemData = (
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, MouseReactive>,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        let mut transform = UiTransform::new(
            self.id.clone(),
            self.anchor.clone(),
            self.x,
            self.y,
            self.z,
            self.width,
            self.height,
            self.tab_order,
        );
        if let Some(ref stretch) = self.stretch {
            transform = transform.with_stretch(stretch.clone());
        }
        if !self.opaque {
            transform = transform.as_transparent();
        }
        if self.percent {
            transform = transform.as_percent();
        }
        system_data.0.insert(entity, transform)?;
        if self.mouse_reactive {
            system_data.1.insert(entity, MouseReactive)?;
        }

        Ok(())
    }
}

/// Loadable `UiText` data
///
/// ### Type parameters:
///
/// - `F`: `Format` used for loading `FontAsset`
#[derive(Deserialize, Serialize)]
pub struct UiTextBuilder<F>
where
    F: Format<FontAsset, Options = ()>,
{
    /// Text to display
    pub text: String,
    /// Font size
    pub font_size: f32,
    /// Font color
    pub color: [f32; 4],
    /// Font
    pub font: AssetPrefab<FontAsset, F>,
    /// Password field ?
    #[serde(default)]
    pub password: bool,
    /// Optionally make the text editable
    #[serde(default)]
    pub editable: Option<TextEditingPrefab>,
}

/// Loadable `TextEditing` data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TextEditingPrefab {
    /// Max number of graphemes
    pub max_length: usize,
    /// Text color on selection
    pub selected_text_color: [f32; 4],
    /// Background color on selection
    pub selected_background_color: [f32; 4],
    /// Use block cursor instead of line cursor
    pub use_block_cursor: bool,
}

impl Default for TextEditingPrefab {
    fn default() -> Self {
        TextEditingPrefab {
            max_length: 20,
            selected_text_color: [0., 0., 0., 1.],
            selected_background_color: [1., 1., 1., 1.],
            use_block_cursor: false,
        }
    }
}

impl<'a, F> PrefabData<'a> for UiTextBuilder<F>
where
    F: Format<FontAsset, Options = ()> + Clone,
{
    type SystemData = (
        WriteStorage<'a, UiText>,
        WriteStorage<'a, TextEditing>,
        <AssetPrefab<FontAsset, F> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        let font_handle = self.font.load_prefab(entity, &mut system_data.2, &[])?;
        let mut ui_text = UiText::new(font_handle, self.text.clone(), self.color, self.font_size);
        ui_text.password = self.password;
        system_data.0.insert(entity, ui_text)?;
        if let Some(ref editing) = self.editable {
            system_data.1.insert(
                entity,
                TextEditing::new(
                    editing.max_length,
                    editing.selected_text_color,
                    editing.selected_background_color,
                    editing.use_block_cursor,
                ),
            )?;
        }
        Ok(())
    }

    fn trigger_sub_loading(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        self.font.trigger_sub_loading(progress, &mut system_data.2)
    }
}

/// Loadable `UiImage` data
///
/// ### Type parameters:
///
/// - `F`: `Format` used for loading `Texture`s
#[derive(Clone, Deserialize, Serialize)]
pub struct UiImageBuilder<F>
where
    F: Format<Texture, Options = TextureMetadata>,
{
    /// Image
    pub image: TexturePrefab<F>,
}

impl<'a, F> PrefabData<'a> for UiImageBuilder<F>
where
    F: Format<Texture, Options = TextureMetadata> + Clone + Sync,
{
    type SystemData = (
        WriteStorage<'a, UiImage>,
        <TexturePrefab<F> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        let texture_handle = self.image
            .load_prefab(entity, &mut system_data.1, entities)?;
        system_data.0.insert(
            entity,
            UiImage {
                texture: texture_handle,
            },
        )?;
        Ok(())
    }

    fn trigger_sub_loading(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        self.image.trigger_sub_loading(progress, &mut system_data.1)
    }
}
