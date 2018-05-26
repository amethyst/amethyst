use amethyst_assets::{AssetPrefab, AssetStorage, Format, Handle, Loader, Prefab, PrefabData,
                      PrefabLoaderSystem, Progress, Result as AssetResult, ResultExt, SimpleFormat};
use amethyst_core::specs::error::Error;
use amethyst_core::specs::prelude::{Entity, Read, ReadExpect, WriteStorage};
use amethyst_renderer::{Texture, TextureMetadata, TexturePrefab};
use serde::Deserialize;

use {Anchor, Anchored, FontAsset, MouseReactive, Stretch, Stretched, TextEditing, UiImage, UiText,
     UiTransform};

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
    stretched: Option<Stretched>,
    anchored: Option<Anchored>,
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
            stretched: None,
            anchored: None,
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
        self.anchored = Some(Anchored::new(anchor));
        self
    }

    /// Set stretch
    pub fn with_stretch(mut self, stretch: Stretch, margin_x: f32, margin_y: f32) -> Self {
        self.stretched = Some(Stretched::new(stretch, margin_x, margin_y));
        self
    }
}

impl<'a> PrefabData<'a> for UiTransformBuilder {
    type SystemData = (
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, MouseReactive>,
        WriteStorage<'a, Stretched>,
        WriteStorage<'a, Anchored>,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), Error> {
        let mut transform = UiTransform::new(
            self.id.clone(),
            self.x,
            self.y,
            self.z,
            self.width,
            self.height,
            self.tab_order,
        );
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
        if let Some(ref stretched) = self.stretched {
            system_data.2.insert(entity, stretched.clone())?;
        }
        if let Some(ref anchored) = self.anchored {
            system_data.3.insert(entity, anchored.clone())?;
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
}

/// Loadable ui components
///
/// ### Type parameters:
///
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
#[derive(Serialize, Deserialize)]
pub enum UiWidget<I, F>
where
    I: Format<Texture, Options = TextureMetadata>,
    F: Format<FontAsset, Options = ()>,
{
    /// Container component, have a transform, an optional background image, and children
    Container(
        UiTransformBuilder,
        Option<UiImageBuilder<I>>,
        Vec<UiWidget<I, F>>,
    ),
    /// Image component, have a transform and an image
    Image(UiTransformBuilder, UiImageBuilder<I>),
    /// Text component, have a transform and text
    Text(UiTransformBuilder, UiTextBuilder<F>),
    /// Button component, have a transform, an image and text
    Button(UiTransformBuilder, UiImageBuilder<I>, UiTextBuilder<F>),
}

type UiPrefabData<I, F> = (
    Option<UiTransformBuilder>,
    Option<UiImageBuilder<I>>,
    Option<UiTextBuilder<F>>,
);

/// Ui prefab
///
/// ### Type parameters:
///
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
pub type UiPrefab<I, F> = Prefab<UiPrefabData<I, F>>;

/// Ui format.
///
/// Load `UiPrefab` from `ron` file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiFormat;

impl<I, F> SimpleFormat<UiPrefab<I, F>> for UiFormat
where
    I: Format<Texture, Options = TextureMetadata> + Sync + for<'a> Deserialize<'a>,
    F: Format<FontAsset, Options = ()> + Sync + for<'a> Deserialize<'a>,
{
    const NAME: &'static str = "Ui";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> AssetResult<UiPrefab<I, F>> {
        use ron::de::Deserializer;
        use serde::Deserialize;
        let mut d = Deserializer::from_bytes(&bytes);
        let root: UiWidget<I, F> =
            UiWidget::deserialize(&mut d).chain_err(|| "Failed parsing Ron file")?;
        d.end().chain_err(|| "Failed parsing Ron file")?;

        let mut prefab = Prefab::new();
        walk_ui_tree(root, None, &mut prefab);

        Ok(prefab)
    }
}

fn walk_ui_tree<I, F>(
    widget: UiWidget<I, F>,
    parent: Option<usize>,
    prefab: &mut Prefab<UiPrefabData<I, F>>,
) where
    I: Format<Texture, Options = TextureMetadata>,
    F: Format<FontAsset, Options = ()>,
{
    match widget {
        UiWidget::Image(transform, image) => {
            prefab.add(parent, Some((Some(transform), Some(image), None)));
        }

        UiWidget::Text(transform, text) => {
            prefab.add(parent, Some((Some(transform), None, Some(text))));
        }

        UiWidget::Button(transform, image, text) => {
            let id = transform.id.clone();
            let current_index = prefab.add(
                parent,
                Some((Some(transform.reactive()), Some(image), None)),
            );
            prefab.add(
                Some(current_index),
                Some((Some(button_text_transform(id)), None, Some(text))),
            );
        }

        UiWidget::Container(transform, image, children) => {
            let current_index = prefab.add(parent, Some((Some(transform), image, None)));
            for child_widget in children {
                walk_ui_tree(child_widget, Some(current_index), prefab);
            }
        }
    }
}

/// Specialised UI loader
///
/// The recommended way of using this in `State`s is with `world.exec`.
///
/// ### Type parameters:
///
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
///
/// ### Example:
///
/// ```rust,ignore
/// let ui_handle = world.exec(|loader: UiLoader<TextureFormat, FontFormat>| {
///     loader.load("ui.ron", ())
/// });
/// ```
#[derive(SystemData)]
pub struct UiLoader<'a, I, F>
where
    I: Format<Texture, Options = TextureMetadata> + Sync,
    F: Format<FontAsset, Options = ()> + Sync,
{
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<UiPrefab<I, F>>>,
}

impl<'a, I, F> UiLoader<'a, I, F>
where
    I: Format<Texture, Options = TextureMetadata> + Sync + for<'b> Deserialize<'b>,
    F: Format<FontAsset, Options = ()> + Sync + for<'b> Deserialize<'b>,
{
    /// Load ui from disc
    pub fn load<N, P>(&self, name: N, progress: P) -> Handle<UiPrefab<I, F>>
    where
        N: Into<String>,
        P: Progress,
    {
        self.loader
            .load(name, UiFormat, (), progress, &self.storage)
    }
}

/// Prefab loader system for UI
///
/// ### Type parameters:
///
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
pub type UiLoaderSystem<I, F> = PrefabLoaderSystem<UiPrefabData<I, F>>;

fn button_text_transform(mut id: String) -> UiTransformBuilder {
    id.push_str("_btn_txt");
    UiTransformBuilder::default()
        .with_id(id)
        .with_position(0., 0., -1.)
        .with_tab_order(10)
        .with_anchor(Anchor::Middle)
        .with_stretch(Stretch::XY, 0., 0.)
        .transparent()
}
