use super::*;
use amethyst_assets::{
    AssetPrefab, AssetStorage, Format, Handle, Loader, Prefab, PrefabData, PrefabError,
    PrefabLoaderSystem, Progress, ProgressCounter, Result as AssetResult, ResultExt, SimpleFormat,
};
use amethyst_audio::{AudioFormat, Source as Audio};
use amethyst_core::specs::prelude::{Entities, Entity, Read, ReadExpect, Write, WriteStorage};
use amethyst_renderer::{Texture, TextureFormat, TextureMetadata, TexturePrefab};
use serde::de::DeserializeOwned;

/// Loadable `UiTransform` data.
/// By default z is equal to one.
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
            z: 1.,
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

    fn add_to_entity(
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
pub struct UiTextBuilder<F = FontFormat>
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
    /// Should the text be shown as dots instead of the proper characters?
    #[serde(default)]
    pub password: bool,
    /// Where should the text be aligned from. Relative to its own UiTransform's area.
    pub align: Option<Anchor>,
    /// How should the text behave with line breaks.
    pub line_mode: Option<LineMode>,
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
    /// Set this text field as focused
    pub focused: bool,
}

impl Default for TextEditingPrefab {
    fn default() -> Self {
        TextEditingPrefab {
            max_length: 20,
            selected_text_color: [0., 0., 0., 1.],
            selected_background_color: [1., 1., 1., 1.],
            use_block_cursor: false,
            focused: false,
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
        Write<'a, UiFocused>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        let (ref mut texts, ref mut editables, ref mut fonts, ref mut focused) = system_data;
        let font_handle = self.font.add_to_entity(entity, fonts, &[])?;
        let mut ui_text = UiText::new(font_handle, self.text.clone(), self.color, self.font_size);
        ui_text.password = self.password;

        if let Some(ref align) = self.align {
            ui_text.align = align.clone();
        }

        if let Some(ref line_mode) = self.line_mode {
            ui_text.line_mode = line_mode.clone();
        }

        texts.insert(entity, ui_text)?;
        if let Some(ref editing) = self.editable {
            editables.insert(
                entity,
                TextEditing::new(
                    editing.max_length,
                    editing.selected_text_color,
                    editing.selected_background_color,
                    editing.use_block_cursor,
                ),
            )?;
            if editing.focused {
                focused.entity = Some(entity);
            }
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let (_, _, ref mut fonts, _) = system_data;
        self.font.load_sub_assets(progress, fonts)
    }
}

/// Loadable `UiImage` data
///
/// ### Type parameters:
///
/// - `F`: `Format` used for loading `Texture`s
#[derive(Clone, Deserialize, Serialize)]
pub struct UiImageBuilder<F = TextureFormat>
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

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        let (ref mut images, ref mut textures) = system_data;
        let texture_handle = self.image.add_to_entity(entity, textures, entities)?;
        images.insert(
            entity,
            UiImage {
                texture: texture_handle,
            },
        )?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let (_, ref mut textures) = system_data;
        self.image.load_sub_assets(progress, textures)
    }
}

/// Loadable `UiButton` data
///
/// ### Type parameters:
///
/// - `TF`: `Format` used for loading `Texture`s
/// - `AF`: `Format` used for loading sounds
#[derive(Deserialize, Serialize)]
pub struct UiButtonBuilder<AF = AudioFormat, TF = TextureFormat, FF = FontFormat>
where
    TF: Format<Texture, Options = TextureMetadata>,
    FF: Format<FontAsset, Options = ()>,
    AF: Format<Audio, Options = ()>,
{
    /// Text to display
    pub text: String,
    /// Font size
    pub font_size: f32,
    /// Font
    pub font: AssetPrefab<FontAsset, FF>,
    /// Default text color
    pub normal_text_color: [f32; 4],
    /// Default image
    pub normal_image: Option<TexturePrefab<TF>>,
    /// Image used when the mouse hovers over this element
    pub hover_image: Option<TexturePrefab<TF>>,
    /// Text color used when this button is hovered over
    pub hover_text_color: Option<[f32; 4]>,
    /// Image used when button is pressed
    pub press_image: Option<TexturePrefab<TF>>,
    /// Text color used when this button is pressed
    pub press_text_color: Option<[f32; 4]>,
    /// Sound made when this button is hovered over
    pub hover_sound: Option<AssetPrefab<Audio, AF>>,
    /// Sound made when this button is pressed.
    pub press_sound: Option<AssetPrefab<Audio, AF>>,
    /// Sound made when this button is released.
    pub release_sound: Option<AssetPrefab<Audio, AF>>,
}

impl<'a, AF, TF, FF> PrefabData<'a> for UiButtonBuilder<AF, TF, FF>
where
    TF: Format<Texture, Options = TextureMetadata> + Clone + Sync,
    FF: Format<FontAsset, Options = ()> + Clone,
    AF: Format<Audio, Options = ()> + Clone,
{
    type SystemData = (
        WriteStorage<'a, UiButton>,
        WriteStorage<'a, OnUiActionImage>,
        WriteStorage<'a, OnUiActionSound>,
        <TexturePrefab<TF> as PrefabData<'a>>::SystemData,
        <AssetPrefab<Audio, AF> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entity_set: &[Entity],
    ) -> Result<(), PrefabError> {
        let (
            ref mut buttons,
            ref mut action_image,
            ref mut action_sound,
            ref mut textures,
            ref mut sounds,
        ) = system_data;
        let normal_image = self
            .normal_image
            .add_to_entity(entity, textures, entity_set)?;
        let hover_image = self
            .hover_image
            .add_to_entity(entity, textures, entity_set)?;
        let press_image = self
            .press_image
            .add_to_entity(entity, textures, entity_set)?;
        let hover_sound = self.hover_sound.add_to_entity(entity, sounds, entity_set)?;
        let press_sound = self.press_sound.add_to_entity(entity, sounds, entity_set)?;
        let release_sound = self
            .release_sound
            .add_to_entity(entity, sounds, entity_set)?;
        buttons.insert(
            entity,
            UiButton::new(
                self.normal_text_color,
                self.hover_text_color,
                self.press_text_color,
            ),
        )?;
        if hover_image.is_some() || press_image.is_some() {
            action_image.insert(
                entity,
                OnUiActionImage::new(normal_image, hover_image, press_image),
            )?;
        }
        if hover_sound.is_some() || press_sound.is_some() || release_sound.is_some() {
            action_sound.insert(
                entity,
                OnUiActionSound::new(hover_sound, press_sound, release_sound),
            )?;
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let (_, _, _, ref mut textures, ref mut sounds) = system_data;
        self.normal_image.load_sub_assets(progress, textures)?;
        self.hover_image.load_sub_assets(progress, textures)?;
        self.press_image.load_sub_assets(progress, textures)?;
        self.press_sound.load_sub_assets(progress, sounds)?;
        self.hover_sound.load_sub_assets(progress, sounds)?;
        self.release_sound.load_sub_assets(progress, sounds)
    }
}

/// Loadable ui components
///
/// ### Type parameters:
///
/// - `A`: `Format` used for loading sounds
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
#[derive(Serialize, Deserialize)]
pub enum UiWidget<A = AudioFormat, I = TextureFormat, F = FontFormat>
where
    A: Format<Audio, Options = ()>,
    I: Format<Texture, Options = TextureMetadata>,
    F: Format<FontAsset, Options = ()>,
{
    /// Container component
    Container {
        /// Spatial information for the container
        transform: UiTransformBuilder,
        /// Background image
        #[serde(default = "default_container_image")]
        background: Option<UiImageBuilder<I>>,
        /// Child widgets
        children: Vec<UiWidget<A, I, F>>,
    },
    /// Image component
    Image {
        /// Spatial information
        transform: UiTransformBuilder,
        /// Image
        image: UiImageBuilder<I>,
    },
    /// Text component
    Text {
        /// Spatial information
        transform: UiTransformBuilder,
        /// Text
        text: UiTextBuilder<F>,
    },
    /// Button component
    Button {
        /// Spatial information
        transform: UiTransformBuilder,
        /// Button
        button: UiButtonBuilder<A, I, F>,
    },
}

fn default_container_image<I>() -> Option<UiImageBuilder<I>>
where
    I: Format<Texture, Options = TextureMetadata>,
{
    None
}

type UiPrefabData<A = AudioFormat, I = TextureFormat, F = FontFormat> = (
    Option<UiTransformBuilder>,
    Option<UiImageBuilder<I>>,
    Option<UiTextBuilder<F>>,
    Option<UiButtonBuilder<A, I, F>>,
);

/// Ui prefab
///
/// ### Type parameters:
///
/// - `A`: `Format` used for loading sounds
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
pub type UiPrefab<A = AudioFormat, I = TextureFormat, F = FontFormat> =
    Prefab<UiPrefabData<A, I, F>>;

/// Ui format.
///
/// Load `UiPrefab` from `ron` file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiFormat;

impl<A, I, F> SimpleFormat<UiPrefab<A, I, F>> for UiFormat
where
    A: Format<Audio, Options = ()> + Sync + DeserializeOwned,
    I: Format<Texture, Options = TextureMetadata> + Sync + DeserializeOwned + Clone,
    F: Format<FontAsset, Options = ()> + Sync + DeserializeOwned + Clone,
{
    const NAME: &'static str = "Ui";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> AssetResult<UiPrefab<A, I, F>> {
        use ron::de::Deserializer;
        use serde::Deserialize;
        let mut d =
            Deserializer::from_bytes(&bytes).chain_err(|| "Failed deserializing Ron file")?;
        let root: UiWidget<A, I, F> =
            UiWidget::deserialize(&mut d).chain_err(|| "Failed parsing Ron file")?;
        d.end().chain_err(|| "Failed parsing Ron file")?;

        let mut prefab = Prefab::new();
        walk_ui_tree(root, 0, &mut prefab);

        Ok(prefab)
    }
}

fn walk_ui_tree<A, I, F>(
    widget: UiWidget<A, I, F>,
    current_index: usize,
    prefab: &mut Prefab<UiPrefabData<A, I, F>>,
) where
    A: Format<Audio, Options = ()>,
    I: Format<Texture, Options = TextureMetadata> + Clone,
    F: Format<FontAsset, Options = ()> + Clone,
{
    match widget {
        UiWidget::Image { transform, image } => {
            prefab.entity(current_index).unwrap().set_data((
                Some(transform),
                Some(image),
                None,
                None,
            ));
        }

        UiWidget::Text { transform, text } => {
            prefab.entity(current_index).unwrap().set_data((
                Some(transform),
                None,
                Some(text),
                None,
            ));
        }

        UiWidget::Container {
            transform,
            background,
            children,
        } => {
            prefab.entity(current_index).unwrap().set_data((
                Some(transform),
                background,
                None,
                None,
            ));
            for child_widget in children {
                let child_index = prefab.add(Some(current_index), None);
                walk_ui_tree(child_widget, child_index, prefab);
            }
        }

        UiWidget::Button { transform, button } => {
            let mut id = transform.id.clone();
            let text = UiTextBuilder {
                color: button.normal_text_color,
                editable: None,
                font: button.font.clone(),
                password: false,
                align: None,
                line_mode: None,
                text: button.text.clone(),
                font_size: button.font_size,
            };
            prefab.entity(current_index).unwrap().set_data((
                Some(transform),
                button.normal_image.as_ref().map(|image| UiImageBuilder {
                    image: image.clone(),
                }),
                None,
                Some(button),
            ));
            prefab.add(
                Some(current_index),
                Some((Some(button_text_transform(id)), None, Some(text), None)),
            );
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
///     loader.load("renderable.ron", ())
/// });
/// ```
#[derive(SystemData)]
pub struct UiLoader<'a, A = AudioFormat, I = TextureFormat, F = FontFormat>
where
    A: Format<Audio, Options = ()> + Sync,
    I: Format<Texture, Options = TextureMetadata> + Sync,
    F: Format<FontAsset, Options = ()> + Sync,
{
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<UiPrefab<A, I, F>>>,
}

impl<'a, A, I, F> UiLoader<'a, A, I, F>
where
    A: Format<Audio, Options = ()> + Sync + DeserializeOwned,
    I: Format<Texture, Options = TextureMetadata> + Sync + DeserializeOwned + Clone,
    F: Format<FontAsset, Options = ()> + Sync + DeserializeOwned + Clone,
{
    /// Load ui from disc
    pub fn load<N, P>(&self, name: N, progress: P) -> Handle<UiPrefab<A, I, F>>
    where
        N: Into<String>,
        P: Progress,
    {
        self.loader
            .load(name, UiFormat, (), progress, &self.storage)
    }
}

/// Ui Creator, wrapper around loading and creating a UI directly.
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
/// let ui_handle = world.exec(|creator: UiCreator| {
///     creator.create("renderable.ron", ())
/// });
/// ```
#[derive(SystemData)]
pub struct UiCreator<'a, A = AudioFormat, I = TextureFormat, F = FontFormat>
where
    A: Format<Audio, Options = ()> + Sync,
    I: Format<Texture, Options = TextureMetadata> + Sync,
    F: Format<FontAsset, Options = ()> + Sync,
{
    loader: UiLoader<'a, A, I, F>,
    entities: Entities<'a>,
    handles: WriteStorage<'a, Handle<UiPrefab<A, I, F>>>,
}

impl<'a, A, I, F> UiCreator<'a, A, I, F>
where
    A: Format<Audio, Options = ()> + Sync + DeserializeOwned + Clone,
    I: Format<Texture, Options = TextureMetadata> + Sync + DeserializeOwned + Clone,
    F: Format<FontAsset, Options = ()> + Sync + DeserializeOwned + Clone,
{
    /// Create a UI.
    ///
    /// Will load a UI from the given `ron` file, create an `Entity` and load the UI with that
    /// `Entity` as the root of the UI hierarchy.
    ///
    /// ### Parameters:
    ///
    /// - `name`: Name of a `ron` asset in the `UiFormat` format
    /// - `progress`: Progress tracker
    ///
    /// ### Returns
    ///
    /// The `Entity` that was created that will form the root of the loaded UI.
    pub fn create<N, P>(&mut self, name: N, progress: P) -> Entity
    where
        N: Into<String>,
        P: Progress,
    {
        let entity = self.entities.create();
        let handle = self.loader.load(name, progress);
        self.handles.insert(entity, handle).unwrap(); // safe because we just created the entity
        entity
    }
}

/// Prefab loader system for UI
///
/// ### Type parameters:
///
/// - `A`: `Format` used for loading sounds
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
pub type UiLoaderSystem<A, I, F> = PrefabLoaderSystem<UiPrefabData<A, I, F>>;

fn button_text_transform(mut id: String) -> UiTransformBuilder {
    id.push_str("_btn_txt");
    UiTransformBuilder::default()
        .with_id(id)
        .with_position(0., 0., 1.)
        .with_tab_order(10)
        .with_anchor(Anchor::Middle)
        .with_stretch(Stretch::XY {
            x_margin: 0.,
            y_margin: 0.,
        }).transparent()
}
