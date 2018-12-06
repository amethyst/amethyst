use serde::de::DeserializeOwned;
use std::marker::PhantomData;

use amethyst_assets::{
    AssetPrefab, AssetStorage, Format, Handle, Loader, Prefab, PrefabData, PrefabError,
    PrefabLoaderSystem, Progress, ProgressCounter, Result as AssetResult, ResultExt, SimpleFormat,
};
use amethyst_audio::{AudioFormat, Source as Audio};
use amethyst_core::specs::{
    error::BoxedErr,
    prelude::{Entities, Entity, Read, ReadExpect, Write, WriteStorage},
};
use amethyst_renderer::{HiddenPropagate, Texture, TextureFormat, TextureMetadata, TexturePrefab};

use super::*;

/// Loadable `UiTransform` data.
/// By default z is equal to one.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct UiTransformBuilder {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Z coordinate, defaults to one.
    pub z: f32,
    /// The width of this UI element.
    pub width: f32,
    /// The height of this UI element.
    pub height: f32,
    /// The UI element tab order.  When the player presses tab the UI focus will shift to the
    /// UI element with the next highest tab order, or if another element with the same tab_order
    /// as this one exists they are ordered according to Entity creation order.  Shift-tab walks
    /// this ordering backwards.
    pub tab_order: i32,
    /// Indicates if actions on the ui can go through this element.
    /// If set to false, the element will behaves as if it was transparent and will let events go to
    /// the next element (for example, the text on a button).
    pub opaque: bool,
    /// Renders this UI element by evaluating transform as a percentage of the parent size,
    /// rather than rendering it with pixel units.
    pub percent: bool,
    /// If a child ui element needs to fill its parent this can be used to stretch it to the appropriate size.
    pub stretch: Option<Stretch>,
    /// Indicates where the element sits, relative to the parent (or to the screen, if there is no parent)
    pub anchor: Anchor,
    /// Allow mouse events on this UI element.
    pub mouse_reactive: bool,
    /// Hides an entity by adding a [`HiddenPropagate`](../amethyst_renderer/struct.HiddenPropagate.html) component
    pub hidden: bool,
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
            hidden: false,
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

    /// Hides an entity by adding a [`HiddenPropagate`](../amethyst_renderer/struct.HiddenPropagate.html) component
    pub fn hide(mut self) -> Self {
        self.hidden = true;
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
        WriteStorage<'a, HiddenPropagate>,
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

        if self.hidden {
            system_data.2.insert(entity, HiddenPropagate)?;
        }

        Ok(())
    }
}

/// Loadable `UiText` data
///
/// ### Type parameters:
///
/// - `F`: `Format` used for loading `FontAsset`
#[derive(Deserialize, Serialize, Clone)]
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
    pub font: Option<AssetPrefab<FontAsset, F>>,
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
        let font_handle = self
            .font
            .as_ref()
            .ok_or_else(|| PrefabError::Custom(BoxedErr(Box::from("did not load sub assets"))))?
            .add_to_entity(entity, fonts, &[])?;
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

        self.font
            .get_or_insert_with(|| {
                let (ref loader, _, ref storage) = fonts;
                AssetPrefab::Handle(get_default_font(loader, storage))
            })
            .load_sub_assets(progress, fonts)
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
#[derive(Deserialize, Serialize, Clone)]
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
    pub font: Option<AssetPrefab<FontAsset, FF>>,
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
#[derive(Serialize, Deserialize, Clone)]
pub enum UiWidget<A = AudioFormat, I = TextureFormat, F = FontFormat, C = NoCustomUi>
where
    A: Format<Audio, Options = ()>,
    I: Format<Texture, Options = TextureMetadata>,
    F: Format<FontAsset, Options = ()>,
    C: ToNativeWidget<A, I, F>,
{
    /// Container component
    Container {
        /// Spatial information for the container
        transform: UiTransformBuilder,
        /// Background image
        #[serde(default = "default_container_image")]
        background: Option<UiImageBuilder<I>>,
        /// Child widgets
        children: Vec<UiWidget<A, I, F, C>>,
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
    /// Custom UI widget
    Custom(Box<C>),
}

impl<A, I, F, C> UiWidget<A, I, F, C>
where
    A: Format<Audio, Options = ()>,
    I: Format<Texture, Options = TextureMetadata>,
    F: Format<FontAsset, Options = ()>,
    C: ToNativeWidget<A, I, F>,
{
    /// Convenience function to access widgets `UiTransformBuilder`
    pub fn transform(&self) -> Option<&UiTransformBuilder> {
        match self {
            UiWidget::Container { ref transform, .. } => Some(transform),
            UiWidget::Image { ref transform, .. } => Some(transform),
            UiWidget::Text { ref transform, .. } => Some(transform),
            UiWidget::Button { ref transform, .. } => Some(transform),
            UiWidget::Custom(_) => None,
        }
    }

    /// Convenience function to access widgets `UiTransformBuilder`
    pub fn transform_mut(&mut self) -> Option<&mut UiTransformBuilder> {
        match self {
            UiWidget::Container {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Image {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Text {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Button {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Custom(_) => None,
        }
    }

    /// Convenience function to access widgets `UiImageBuilder`
    pub fn image(&self) -> Option<&UiImageBuilder<I>> {
        match self {
            UiWidget::Container { ref background, .. } => background.as_ref(),
            UiWidget::Image { ref image, .. } => Some(image),
            _ => None,
        }
    }

    /// Convenience function to access widgets `UiImageBuilder`
    pub fn image_mut(&mut self) -> Option<&mut UiImageBuilder<I>> {
        match self {
            UiWidget::Container {
                ref mut background, ..
            } => background.as_mut(),
            UiWidget::Image { ref mut image, .. } => Some(image),
            _ => None,
        }
    }
}

/// Create native `UiWidget` from custom UI
pub trait ToNativeWidget<A = AudioFormat, I = TextureFormat, F = FontFormat>
where
    A: Format<Audio, Options = ()>,
    I: Format<Texture, Options = TextureMetadata>,
    F: Format<FontAsset, Options = ()>,
    Self: Sized,
{
    /// Additional data used when loading UI prefab
    type PrefabData: for<'a> PrefabData<'a> + Default + Send + Sync + 'static;
    /// Create native `UiWidget` and custom prefab data from custom UI
    ///
    /// Returning `UiWidget::Custom` will cause recursion.
    /// Please make sure that the recursion is finite.
    fn to_native_widget(
        self,
        parent_data: Self::PrefabData,
    ) -> (UiWidget<A, I, F, Self>, Self::PrefabData);
}

/// Type used when no custom ui is desired
#[derive(Serialize, Deserialize)]
pub enum NoCustomUi {}

impl<A, I, F> ToNativeWidget<A, I, F> for NoCustomUi
where
    A: Format<Audio, Options = ()>,
    I: Format<Texture, Options = TextureMetadata>,
    F: Format<FontAsset, Options = ()>,
{
    type PrefabData = ();
    fn to_native_widget(
        self,
        _: Self::PrefabData,
    ) -> (UiWidget<A, I, F, NoCustomUi>, Self::PrefabData) {
        // self can not exist
        unreachable!()
    }
}

fn default_container_image<I>() -> Option<UiImageBuilder<I>>
where
    I: Format<Texture, Options = TextureMetadata>,
{
    None
}

type UiPrefabData<
    A = AudioFormat,
    I = TextureFormat,
    F = FontFormat,
    D = <NoCustomUi as ToNativeWidget>::PrefabData,
> = (
    Option<UiTransformBuilder>,
    Option<UiImageBuilder<I>>,
    Option<UiTextBuilder<F>>,
    Option<UiButtonBuilder<A, I, F>>,
    D,
);

/// Ui prefab
///
/// ### Type parameters:
///
/// - `A`: `Format` used for loading sounds
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
/// - `D`: `ToNativeWidget::PrefabData` data used by custom UI
pub type UiPrefab<
    A = AudioFormat,
    I = TextureFormat,
    F = FontFormat,
    D = <NoCustomUi as ToNativeWidget>::PrefabData,
> = Prefab<UiPrefabData<A, I, F, D>>;

/// Ui format.
///
/// Load `UiPrefab` from `ron` file.
#[derive(Serialize, Deserialize, Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""), Default(bound = ""))]
pub struct UiFormat<C>(PhantomData<C>);

impl<A, I, F, C> SimpleFormat<UiPrefab<A, I, F, C::PrefabData>> for UiFormat<C>
where
    A: Format<Audio, Options = ()> + Sync + DeserializeOwned,
    I: Format<Texture, Options = TextureMetadata> + Sync + DeserializeOwned + Clone,
    F: Format<FontAsset, Options = ()> + Sync + DeserializeOwned + Clone,
    C: ToNativeWidget<A, I, F> + for<'de> serde::Deserialize<'de>,
{
    const NAME: &'static str = "Ui";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> AssetResult<UiPrefab<A, I, F, C::PrefabData>> {
        use ron::de::Deserializer;
        use serde::Deserialize;
        let mut d =
            Deserializer::from_bytes(&bytes).chain_err(|| "Failed deserializing Ron file")?;
        let root: UiWidget<A, I, F, C> =
            UiWidget::deserialize(&mut d).chain_err(|| "Failed parsing Ron file")?;
        d.end().chain_err(|| "Failed parsing Ron file")?;

        let mut prefab = Prefab::new();
        walk_ui_tree(root, 0, &mut prefab, Default::default());

        Ok(prefab)
    }
}

fn walk_ui_tree<A, I, F, C>(
    widget: UiWidget<A, I, F, C>,
    current_index: usize,
    prefab: &mut Prefab<UiPrefabData<A, I, F, C::PrefabData>>,
    custom_data: C::PrefabData,
) where
    A: Format<Audio, Options = ()>,
    I: Format<Texture, Options = TextureMetadata> + Clone,
    F: Format<FontAsset, Options = ()> + Clone,
    C: ToNativeWidget<A, I, F>,
{
    match widget {
        UiWidget::Custom(custom) => {
            let (widget, custom_data) = custom.to_native_widget(custom_data);
            walk_ui_tree(widget, current_index, prefab, custom_data);
        }

        UiWidget::Image { transform, image } => {
            prefab
                .entity(current_index)
                .expect("Unreachable: `Prefab` entity should always be set when walking ui tree")
                .set_data((Some(transform), Some(image), None, None, custom_data));
        }

        UiWidget::Text { transform, text } => {
            prefab
                .entity(current_index)
                .expect("Unreachable: `Prefab` entity should always be set when walking ui tree")
                .set_data((Some(transform), None, Some(text), None, custom_data));
        }

        UiWidget::Container {
            transform,
            background,
            children,
        } => {
            prefab
                .entity(current_index)
                .expect("Unreachable: `Prefab` entity should always be set when walking ui tree")
                .set_data((Some(transform), background, None, None, custom_data));

            for child_widget in children {
                let child_index = prefab.add(Some(current_index), None);
                walk_ui_tree(child_widget, child_index, prefab, Default::default());
            }
        }

        UiWidget::Button { transform, button } => {
            let id = transform.id.clone();
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

            prefab
                .entity(current_index)
                .expect("Unreachable: `Prefab` entity should always be set when walking ui tree")
                .set_data((
                    Some(transform),
                    button.normal_image.as_ref().map(|image| UiImageBuilder {
                        image: image.clone(),
                    }),
                    None,
                    Some(button),
                    custom_data,
                ));

            prefab.add(
                Some(current_index),
                Some((
                    Some(button_text_transform(id)),
                    None,
                    Some(text),
                    None,
                    Default::default(),
                )),
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
pub struct UiLoader<'a, A = AudioFormat, I = TextureFormat, F = FontFormat, C = NoCustomUi>
where
    A: Format<Audio, Options = ()> + Sync,
    I: Format<Texture, Options = TextureMetadata> + Sync,
    F: Format<FontAsset, Options = ()> + Sync,
    C: ToNativeWidget<A, I, F>,
{
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<UiPrefab<A, I, F, C::PrefabData>>>,
}

impl<'a, A, I, F, C> UiLoader<'a, A, I, F, C>
where
    A: Format<Audio, Options = ()> + Sync + DeserializeOwned,
    I: Format<Texture, Options = TextureMetadata> + Sync + DeserializeOwned + Clone,
    F: Format<FontAsset, Options = ()> + Sync + DeserializeOwned + Clone,
    C: ToNativeWidget<A, I, F> + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
{
    /// Load ui from disc
    pub fn load<N, P>(&self, name: N, progress: P) -> Handle<UiPrefab<A, I, F, C::PrefabData>>
    where
        N: Into<String>,
        P: Progress,
    {
        self.loader
            .load(name, UiFormat::<C>::default(), (), progress, &self.storage)
    }
}

/// Ui Creator, wrapper around loading and creating a UI directly.
///
/// The recommended way of using this in `State`s is with `world.exec`.
///
/// ### Type parameters:
///
/// - `A`: `Format` used for loading sounds
/// - `I`: `Format` used for loading `Texture`s
/// - `F`: `Format` used for loading `FontAsset`
/// - `C`: custom UI widget
///
/// ### Example:
///
/// ```rust,ignore
/// let ui_handle = world.exec(|creator: UiCreator| {
///     creator.create("renderable.ron", ())
/// });
/// ```
#[derive(SystemData)]
pub struct UiCreator<'a, A = AudioFormat, I = TextureFormat, F = FontFormat, C = NoCustomUi>
where
    A: Format<Audio, Options = ()> + Sync,
    I: Format<Texture, Options = TextureMetadata> + Sync,
    F: Format<FontAsset, Options = ()> + Sync,
    C: ToNativeWidget<A, I, F>,
{
    loader: UiLoader<'a, A, I, F, C>,
    entities: Entities<'a>,
    handles: WriteStorage<'a, Handle<UiPrefab<A, I, F, C::PrefabData>>>,
}

impl<'a, A, I, F, C> UiCreator<'a, A, I, F, C>
where
    A: Format<Audio, Options = ()> + Sync + DeserializeOwned + Clone,
    I: Format<Texture, Options = TextureMetadata> + Sync + DeserializeOwned + Clone,
    F: Format<FontAsset, Options = ()> + Sync + DeserializeOwned + Clone,
    C: ToNativeWidget<A, I, F> + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
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
        self.handles
            .insert(entity, handle)
            .expect("Unreachable: We just created the entity");
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
/// - `CD`: prefab data from custom UI, see `ToNativeWidget::PrefabData`
pub type UiLoaderSystem<A, I, F, CD> = PrefabLoaderSystem<UiPrefabData<A, I, F, CD>>;

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
        })
        .transparent()
}
