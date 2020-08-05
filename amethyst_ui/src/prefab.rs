use derivative::Derivative;
use serde::de::DeserializeOwned;
use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use amethyst_assets::{
    AssetPrefab, AssetStorage, Format, Handle, Loader, Prefab, PrefabData, PrefabLoaderSystem,
    PrefabLoaderSystemDesc, Progress, ProgressCounter,
};
use amethyst_audio::Source as Audio;
use amethyst_core::{
    ecs::{
        prelude::{Entities, Entity, Read, ReadExpect, World, Write, WriteStorage},
        shred::{ResourceId, SystemData},
    },
    HiddenPropagate,
};
use amethyst_error::{format_err, Error, ResultExt};
use amethyst_rendy::TexturePrefab;

use serde::{Deserialize, Serialize};

use crate::{
    get_default_font, Anchor, Draggable, FontAsset, Interactable, LineMode, Selectable, Stretch,
    TextEditing, UiButton, UiButtonAction, UiButtonActionRetrigger, UiButtonActionType, UiImage,
    UiPlaySoundAction, UiSoundRetrigger, UiText, UiTransform, WidgetId, Widgets,
};

/// Loadable `UiTransform` data.
/// By default z is equal to one.
#[derive(Debug, Clone, Deserialize, Serialize, Derivative)]
#[serde(default)]
#[derivative(Default)]
pub struct UiTransformData<G> {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    #[derivative(Default(value = "1.0"))]
    /// Z coordinate, defaults to one.
    pub z: f32,
    /// The width of this UI element.
    pub width: f32,
    /// The height of this UI element.
    pub height: f32,
    /// Indicates if actions on the ui can go through this element.
    /// If set to false, the element will behaves as if it was transparent and will let events go to
    /// the next element (for example, the text on a button).
    #[derivative(Default(value = "true"))]
    pub opaque: bool,
    /// Allows transparent (`opaque = false`) transforms to still be targeted by the events that
    /// pass through them.
    #[derivative(Default(value = "false"))]
    pub transparent_target: bool,
    /// Renders this UI element by evaluating transform as a percentage of the parent size,
    /// rather than rendering it with pixel units.
    pub percent: bool,
    /// If a child ui element needs to fill its parent this can be used to stretch it to the appropriate size.
    pub stretch: Option<Stretch>,
    /// Indicates where the element sits, relative to the parent (or to the screen, if there is no parent)
    #[derivative(Default(value = "Anchor::Middle"))]
    pub anchor: Anchor,
    /// Indicates where the element sits, relative to itself
    #[derivative(Default(value = "Anchor::Middle"))]
    pub pivot: Anchor,
    /// Allow mouse events on this UI element.
    pub mouse_reactive: bool,
    /// Hides an entity by adding a [`HiddenPropagate`](../amethyst_renderer/struct.HiddenPropagate.html) component
    pub hidden: bool,
    /// Makes the UiTransform selectable through keyboard inputs, mouse inputs and other means.
    /// # Ordering
    /// The UI element tab order.  When the player presses tab the UI focus will shift to the
    /// UI element with the next highest tab order, or if another element with the same tab_order
    /// as this one exists they are ordered according to Entity creation order.  Shift-tab walks
    /// this ordering backwards.
    // TODO: Make full prefab for Selectable.
    pub selectable: Option<u32>,
    /// Makes the UiTransform draggable through mouse inputs.
    pub draggable: bool,
    #[serde(skip)]
    _phantom: PhantomData<G>,
}

impl<G> UiTransformData<G> {
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

impl<'a, G> PrefabData<'a> for UiTransformData<G>
where
    G: Send + Sync + 'static,
{
    type SystemData = (
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, Interactable>,
        WriteStorage<'a, HiddenPropagate>,
        WriteStorage<'a, Selectable<G>>,
        WriteStorage<'a, Draggable>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        let mut transform = UiTransform::new(
            self.id.clone(),
            self.anchor,
            self.pivot,
            self.x,
            self.y,
            self.z,
            self.width,
            self.height,
        );
        if let Some(ref stretch) = self.stretch {
            transform = transform.with_stretch(stretch.clone());
        }
        if !self.opaque {
            transform = transform.into_transparent();
        }
        transform.transparent_target = self.transparent_target;
        if self.percent {
            transform = transform.into_percent();
        }
        system_data.0.insert(entity, transform)?;
        if self.mouse_reactive {
            system_data.1.insert(entity, Interactable)?;
        }

        if self.hidden {
            system_data.2.insert(entity, HiddenPropagate::new())?;
        }

        if let Some(u) = self.selectable {
            system_data.3.insert(entity, Selectable::<G>::new(u))?;
        }

        if self.draggable {
            system_data.4.insert(entity, Draggable)?;
        }

        Ok(())
    }
}

/// Loadable `UiText` data
///
/// ### Type parameters:
///
/// - `F`: `Format` used for loading `FontAsset`
/// - `G`: Selection Group Type
#[derive(Deserialize, Serialize, Clone)]
pub struct UiTextData {
    /// Text to display
    pub text: String,
    /// Font size
    pub font_size: f32,
    /// Font color
    pub color: [f32; 4],
    /// Font
    pub font: Option<AssetPrefab<FontAsset>>,
    /// Should the text be shown as dots instead of the proper characters?
    #[serde(default)]
    pub password: bool,
    /// How should the text behave with line breaks.
    pub line_mode: Option<LineMode>,
    /// Where should the text be aligned from. Relative to its own UiTransform's area.
    pub align: Option<Anchor>,
    /// Optionally make the text editable
    #[serde(default)]
    pub editable: Option<TextEditingPrefab>,
}
impl Debug for UiTextData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let font = match self.font.as_ref() {
            Some(asset_prefab) => match asset_prefab {
                AssetPrefab::File(path, _) => format!("<Font:{}>", path),
                _ => "<Font>".to_string(),
            },
            _ => "<Font>".to_string(),
        };

        f.debug_struct("UiTextData")
            .field("text", &self.text)
            .field("font_size", &self.font_size)
            .field("font", &font)
            .field("color", &self.color)
            .field("password", &self.password)
            .field("line_mode", &self.line_mode)
            .field("align", &self.align)
            .field("editable", &self.editable)
            .finish()
    }
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

impl<'a> PrefabData<'a> for UiTextData {
    type SystemData = (
        WriteStorage<'a, UiText>,
        WriteStorage<'a, TextEditing>,
        <AssetPrefab<FontAsset> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        let (ref mut texts, ref mut editables, ref mut fonts) = system_data;
        let font_handle = self
            .font
            .as_ref()
            .ok_or_else(|| format_err!("did not load sub assets"))?
            .add_to_entity(entity, fonts, &[], &[])?;

        let mut ui_text_align = Anchor::Middle;
        if let Some(align) = self.align {
            ui_text_align = align;
        }
        let mut ui_text_line_mode = LineMode::Single;
        if let Some(line_mode) = self.line_mode {
            ui_text_line_mode = line_mode;
        }

        let mut ui_text = UiText::new(
            font_handle,
            self.text.clone(),
            self.color,
            self.font_size,
            ui_text_line_mode,
            ui_text_align,
        );
        ui_text.password = self.password;

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
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (_, _, ref mut fonts) = system_data;

        self.font
            .get_or_insert_with(|| {
                let (ref loader, _, ref storage) = fonts;
                AssetPrefab::Handle(get_default_font(loader, storage))
            })
            .load_sub_assets(progress, fonts)
    }
}

/// Loadable `UiImage` data. Adds UiImage component to the entity.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct UiImagePrefab(pub UiImageLoadPrefab);

/// Loadable `UiImage` data. Returns image component from `add_to_entity` instead of adding it.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "UiImagePrefab")]
pub enum UiImageLoadPrefab {
    /// A textured image
    Texture(TexturePrefab),
    /// A partial textured image
    ///
    /// Coordinates are texture coordinates -- between `0.0` and `1.0` inclusive.
    PartialTexture {
        /// Texture prefab.
        tex: TexturePrefab,
        /// Coordinate of the left edge of the texture.
        left: f32,
        /// Coordinate of the right edge of the texture.
        right: f32,
        /// Coordinate of the bottom edge of the texture.
        bottom: f32,
        /// Coordinate of the top edge of the texture.
        top: f32,
    },
    /// Solid color image
    SolidColor(f32, f32, f32, f32),
    /// 9-Slice image
    ///
    /// Coordinates are in pixels.
    NineSlice {
        /// Coordinate of the left edge of the left slice.
        x_start: u32,
        /// Coordinate of the top edge of the top slice.
        y_start: u32,
        /// Width of the nine slice (exclude padding pixels).
        width: u32,
        /// Height of the nine slice (exclude padding pixels).
        height: u32,
        /// Width of the left slice.
        left_dist: u32,
        /// Width of the right slice.
        right_dist: u32,
        /// Height of the top slice.
        top_dist: u32,
        /// Height of the bottom slice.
        bottom_dist: u32,
        /// Texture prefab.
        tex: TexturePrefab,
        /// Texture dimensions.
        texture_dimensions: (u32, u32),
    },
}

impl<'a> PrefabData<'a> for UiImagePrefab {
    type SystemData = (
        <UiImageLoadPrefab as PrefabData<'a>>::SystemData,
        WriteStorage<'a, UiImage>,
    );

    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (ref mut inner, ref mut images): &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        let image = self.0.add_to_entity(entity, inner, entities, children)?;
        images.insert(entity, image)?;
        Ok(())
    }
    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        (ref mut inner, _): &mut Self::SystemData,
    ) -> Result<bool, Error> {
        self.0.load_sub_assets(progress, inner)
    }
}

impl<'a> PrefabData<'a> for UiImageLoadPrefab {
    type SystemData = <TexturePrefab as PrefabData<'a>>::SystemData;
    type Result = UiImage;

    fn add_to_entity(
        &self,
        entity: Entity,
        textures: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<UiImage, Error> {
        let image = match self {
            UiImageLoadPrefab::Texture(tex) => {
                UiImage::Texture(tex.add_to_entity(entity, textures, entities, children)?)
            }
            UiImageLoadPrefab::PartialTexture {
                tex,
                left,
                right,
                bottom,
                top,
            } => UiImage::PartialTexture {
                tex: tex.add_to_entity(entity, textures, entities, children)?,
                left: *left,
                right: *right,
                bottom: *bottom,
                top: *top,
            },
            UiImageLoadPrefab::NineSlice {
                x_start,
                y_start,
                height,
                width,
                left_dist,
                right_dist,
                top_dist,
                bottom_dist,
                tex,
                texture_dimensions,
            } => UiImage::NineSlice {
                x_start: *x_start,
                y_start: *y_start,
                height: *height,
                width: *width,
                left_dist: *left_dist,
                right_dist: *right_dist,
                top_dist: *top_dist,
                bottom_dist: *bottom_dist,
                tex: tex.add_to_entity(entity, textures, entities, children)?,
                texture_dimensions: [texture_dimensions.0, texture_dimensions.1],
            },
            UiImageLoadPrefab::SolidColor(r, g, b, a) => UiImage::SolidColor([*r, *g, *b, *a]),
        };
        Ok(image)
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        textures: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        match self {
            UiImageLoadPrefab::Texture(tex) => tex.load_sub_assets(progress, textures),
            UiImageLoadPrefab::PartialTexture { tex, .. } => {
                tex.load_sub_assets(progress, textures)
            }
            UiImageLoadPrefab::NineSlice { tex, .. } => tex.load_sub_assets(progress, textures),
            UiImageLoadPrefab::SolidColor(..) => Ok(false),
        }
    }
}

/// Loadable `UiButton` data
///
/// ### Type parameters:
///
/// - `W`: Type used for Widget IDs
#[derive(Deserialize, Serialize, Clone)]
pub struct UiButtonData<W: WidgetId = u32> {
    /// Id for the widget
    pub id: Option<W>,
    /// Text to display
    pub text: String,
    /// Font size
    pub font_size: f32,
    /// Font
    pub font: Option<AssetPrefab<FontAsset>>,
    /// Default text color
    pub normal_text_color: [f32; 4],
    // this `normal_image` is "transplanted" into UiImagePrefab at the top level
    // of ui widegt. This happens inside `walk_ui_tree` function. It means that
    // it will always be `None` during `add_to_entity`.
    /// Default image
    pub normal_image: Option<UiImageLoadPrefab>,
    /// Image used when the mouse hovers over this element
    pub hover_image: Option<UiImageLoadPrefab>,
    /// Text color used when this button is hovered over
    pub hover_text_color: Option<[f32; 4]>,
    /// Image used when button is pressed
    pub press_image: Option<UiImageLoadPrefab>,
    /// Text color used when this button is pressed
    pub press_text_color: Option<[f32; 4]>,
    /// Sound made when this button is hovered over
    pub hover_sound: Option<AssetPrefab<Audio>>,
    /// Sound made when this button is pressed.
    pub press_sound: Option<AssetPrefab<Audio>>,
    /// Sound made when this button is released.
    pub release_sound: Option<AssetPrefab<Audio>>,
}

impl<W: WidgetId + Debug> Debug for UiButtonData<W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let font = match self.font.as_ref() {
            Some(asset_prefab) => match asset_prefab {
                AssetPrefab::File(path, _) => format!("<Font:{}>", path),
                _ => "<Font>".to_string(),
            },
            _ => "<Font>".to_string(),
        };

        f.debug_struct("UiTextData")
            .field("id", &self.id)
            .field("text", &self.text)
            .field("font_size", &self.font_size)
            .field("font", &font)
            .field("normal_text_color", &self.normal_text_color)
            .field("normal_image", &self.normal_image)
            .field("hover_image", &self.hover_image)
            .field("press_image", &self.press_image)
            .field("press_text_color", &self.press_text_color)
            .field("hover_sound", &self.hover_sound)
            .field("press_sound", &self.press_sound)
            .field("release_sound", &self.release_sound)
            .finish()
    }
}

impl<'a, W> PrefabData<'a> for UiButtonData<W>
where
    W: WidgetId,
{
    type SystemData = (
        WriteStorage<'a, UiSoundRetrigger>,
        WriteStorage<'a, UiButtonActionRetrigger>,
        Write<'a, Widgets<UiButton, W>>,
        <UiImageLoadPrefab as PrefabData<'a>>::SystemData,
        <AssetPrefab<Audio> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entity_set: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        let (
            ref mut sound_retrigger,
            ref mut button_action_retrigger,
            ref mut widgets,
            ref mut images,
            ref mut sounds,
        ) = system_data;

        let text_entity = children.get(0).expect("Invalid: Should have text child");
        let widget = UiButton::new(entity, *text_entity);
        if let Some(id) = &self.id {
            widgets.add_with_id(id.clone(), widget);
        } else {
            widgets.add(widget);
        }

        let _normal_image = self
            .normal_image
            .add_to_entity(entity, images, entity_set, children)?;
        let hover_image = self
            .hover_image
            .add_to_entity(entity, images, entity_set, children)?;
        let press_image = self
            .press_image
            .add_to_entity(entity, images, entity_set, children)?;

        let hover_sound = self
            .hover_sound
            .add_to_entity(entity, sounds, entity_set, children)?;
        let press_sound = self
            .press_sound
            .add_to_entity(entity, sounds, entity_set, children)?;
        let release_sound = self
            .release_sound
            .add_to_entity(entity, sounds, entity_set, children)?;

        let mut on_click_start = Vec::new();
        let mut on_click_stop = Vec::new();
        let mut on_hover_start = Vec::new();
        let mut on_hover_stop = Vec::new();

        if let Some(press_image) = press_image {
            on_click_start.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::SetImage(press_image.clone()),
            });

            on_click_stop.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::UnsetTexture(press_image),
            });
        }

        if let Some(hover_image) = hover_image {
            on_hover_start.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::SetImage(hover_image.clone()),
            });

            on_hover_stop.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::UnsetTexture(hover_image),
            });
        }

        if let Some(press_text_color) = self.press_text_color {
            on_click_start.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::SetTextColor(press_text_color),
            });

            on_click_stop.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::UnsetTextColor(press_text_color),
            });
        }

        if let Some(hover_text_color) = self.hover_text_color {
            on_hover_start.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::SetTextColor(hover_text_color),
            });

            on_hover_stop.push(UiButtonAction {
                target: entity,
                event_type: UiButtonActionType::UnsetTextColor(hover_text_color),
            });
        }

        if !on_click_start.is_empty()
            || !on_click_stop.is_empty()
            || !on_hover_start.is_empty()
            || !on_hover_stop.is_empty()
        {
            let retrigger = UiButtonActionRetrigger {
                on_click_start,
                on_click_stop,
                on_hover_start,
                on_hover_stop,
            };

            button_action_retrigger.insert(entity, retrigger)?;
        }

        if hover_sound.is_some() || press_sound.is_some() || release_sound.is_some() {
            let retrigger = UiSoundRetrigger {
                on_click_start: press_sound.map(UiPlaySoundAction),
                on_click_stop: release_sound.map(UiPlaySoundAction),
                on_hover_start: hover_sound.map(UiPlaySoundAction),
                on_hover_stop: None,
            };

            sound_retrigger.insert(entity, retrigger)?;
        }

        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (_, _, _, ref mut images, ref mut sounds) = system_data;
        self.normal_image.load_sub_assets(progress, images)?;
        self.hover_image.load_sub_assets(progress, images)?;
        self.press_image.load_sub_assets(progress, images)?;
        self.press_sound.load_sub_assets(progress, sounds)?;
        self.hover_sound.load_sub_assets(progress, sounds)?;
        self.release_sound.load_sub_assets(progress, sounds)
    }
}

/// Loadable ui components
///
/// ### Type parameters:
///
/// - `W`: Type used for Widget ID for this widget and its children
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(clippy::large_enum_variant)] // TODO: revisit this for actual memory usage optimization
pub enum UiWidget<C = NoCustomUi, W = u32, G = ()>
where
    C: ToNativeWidget<W>,
    W: WidgetId,
{
    /// Container widget
    Container {
        /// Spatial information for the container
        transform: UiTransformData<G>,
        /// Background image
        #[serde(default = "default_container_image")]
        background: Option<UiImagePrefab>,
        /// Child widgets
        children: Vec<UiWidget<C, W>>,
    },
    /// Image widget
    Image {
        /// Spatial information
        transform: UiTransformData<G>,
        /// Image
        image: UiImagePrefab,
    },
    /// Text widget
    Label {
        /// Spatial information
        transform: UiTransformData<G>,
        /// Text
        text: UiTextData,
    },
    /// Button widget
    Button {
        /// Spatial information
        transform: UiTransformData<G>,
        /// Button
        button: UiButtonData<W>,
    },
    /// Custom UI widget
    Custom(Box<C>),
}

impl<C, W, G> UiWidget<C, W, G>
where
    C: ToNativeWidget<W>,
    W: WidgetId,
{
    /// Convenience function to access widgets `UiTransformData`
    pub fn transform(&self) -> Option<&UiTransformData<G>> {
        match self {
            UiWidget::Container { ref transform, .. } => Some(transform),
            UiWidget::Image { ref transform, .. } => Some(transform),
            UiWidget::Label { ref transform, .. } => Some(transform),
            UiWidget::Button { ref transform, .. } => Some(transform),
            UiWidget::Custom(_) => None,
        }
    }

    /// Convenience function to access widgets `UiTransformData`
    pub fn transform_mut(&mut self) -> Option<&mut UiTransformData<G>> {
        match self {
            UiWidget::Container {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Image {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Label {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Button {
                ref mut transform, ..
            } => Some(transform),
            UiWidget::Custom(_) => None,
        }
    }

    /// Convenience function to access widgets `UiImagePrefab`
    pub fn image(&self) -> Option<&UiImagePrefab> {
        match self {
            UiWidget::Container { ref background, .. } => background.as_ref(),
            UiWidget::Image { ref image, .. } => Some(image),
            _ => None,
        }
    }

    /// Convenience function to access widgets `UiImagePrefab`
    pub fn image_mut(&mut self) -> Option<&mut UiImagePrefab> {
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
pub trait ToNativeWidget<W = u32>
where
    W: WidgetId,
    Self: Sized + 'static,
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
    ) -> (UiWidget<Self, W>, Self::PrefabData);
}

/// Type used when no custom ui is desired
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum NoCustomUi {}

impl<W> ToNativeWidget<W> for NoCustomUi
where
    W: WidgetId,
{
    type PrefabData = ();
    fn to_native_widget(self, _: Self::PrefabData) -> (UiWidget<NoCustomUi, W>, Self::PrefabData) {
        // self can not exist
        unreachable!()
    }
}

fn default_container_image() -> Option<UiImagePrefab> {
    None
}

type UiPrefabData<D = <NoCustomUi as ToNativeWidget>::PrefabData, W = u32, G = ()> = (
    Option<UiTransformData<G>>,
    Option<UiImagePrefab>,
    Option<UiTextData>,
    Option<UiButtonData<W>>,
    D,
);

/// Ui prefab
///
/// ### Type parameters:
///
/// - `D`: `ToNativeWidget::PrefabData` data used by custom UI
/// - `W`: Type used for Widget IDs
pub type UiPrefab<D = <NoCustomUi as ToNativeWidget>::PrefabData, W = u32> =
    Prefab<UiPrefabData<D, W>>;

/// Ui format.
///
/// Load `UiPrefab` from `ron` file.
#[derive(Serialize, Deserialize, Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""), Default(bound = ""))]
pub struct UiFormat<C>(PhantomData<C>);

unsafe impl<C> Send for UiFormat<C> {}
unsafe impl<C> Sync for UiFormat<C> {}

impl<C, W> Format<UiPrefab<C::PrefabData, W>> for UiFormat<C>
where
    C: ToNativeWidget<W> + for<'de> serde::Deserialize<'de>,
    W: WidgetId + DeserializeOwned,
{
    fn name(&self) -> &'static str {
        "Ui"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<UiPrefab<C::PrefabData, W>, Error> {
        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&bytes)
            .with_context(|_| format_err!("Failed deserializing Ron file"))?;
        let root: UiWidget<C, W> = UiWidget::deserialize(&mut d)
            .with_context(|_| format_err!("Failed parsing Ron file"))?;
        d.end()
            .with_context(|_| format_err!("Failed parsing Ron file"))?;

        let mut prefab = Prefab::new();
        walk_ui_tree(root, 0, &mut prefab, Default::default());

        Ok(prefab)
    }
}

fn walk_ui_tree<C, W>(
    widget: UiWidget<C, W>,
    current_index: usize,
    prefab: &mut Prefab<UiPrefabData<C::PrefabData, W>>,
    custom_data: C::PrefabData,
) where
    C: ToNativeWidget<W>,
    W: WidgetId,
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

        UiWidget::Label { transform, text } => {
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

        UiWidget::Button {
            transform,
            mut button,
        } => {
            let id = transform.id.clone();
            let text = UiTextData {
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
                    button.normal_image.take().map(UiImagePrefab),
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
/// - `W`: Type used for Widget IDs
///
/// ### Example:
///
/// ```rust,ignore
/// let ui_handle = world.exec(|loader: UiLoader<TextureFormat, FontFormat>| {
///     loader.load("renderable.ron", ())
/// });
/// ```
#[derive(SystemData)]
#[allow(missing_debug_implementations)]
pub struct UiLoader<'a, C = NoCustomUi, W = u32>
where
    C: ToNativeWidget<W>,
    W: WidgetId,
{
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<UiPrefab<C::PrefabData, W>>>,
}

impl<'a, C, W> UiLoader<'a, C, W>
where
    C: ToNativeWidget<W> + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
    W: WidgetId + DeserializeOwned,
{
    /// Load ui from disc
    pub fn load<N, P>(&self, name: N, progress: P) -> Handle<UiPrefab<C::PrefabData, W>>
    where
        N: Into<String>,
        P: Progress,
    {
        self.loader
            .load(name, UiFormat::<C>::default(), progress, &self.storage)
    }
}

/// Ui Creator, wrapper around loading and creating a UI directly.
///
/// The recommended way of using this in `State`s is with `world.exec`.
///
/// ### Type parameters:
///
/// - `C`: custom UI widget
/// - `W`: Type used for Widget IDs
///
/// ### Example:
///
/// ```rust,ignore
/// let ui_handle = world.exec(|creator: UiCreator| {
///     creator.create("renderable.ron", ())
/// });
/// ```
#[derive(SystemData)]
#[allow(missing_debug_implementations)]
pub struct UiCreator<'a, C = NoCustomUi, W = u32>
where
    C: ToNativeWidget<W>,
    W: WidgetId,
{
    loader: UiLoader<'a, C, W>,
    entities: Entities<'a>,
    handles: WriteStorage<'a, Handle<UiPrefab<C::PrefabData, W>>>,
}

impl<'a, C, W> UiCreator<'a, C, W>
where
    C: ToNativeWidget<W> + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
    W: WidgetId + DeserializeOwned,
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

/// Builds a `UiLoaderSystem`.
pub type UiLoaderSystemDesc<CD, W> = PrefabLoaderSystemDesc<UiPrefabData<CD, W>>;

/// Prefab loader system for UI
///
/// ### Type parameters:
///
/// - `CD`: prefab data from custom UI, see `ToNativeWidget::PrefabData`
/// - `W`: Type used for Widget IDs
pub type UiLoaderSystem<CD, W> = PrefabLoaderSystem<UiPrefabData<CD, W>>;

fn button_text_transform<G>(mut id: String) -> UiTransformData<G> {
    id.push_str("_btn_txt");
    UiTransformData::default()
        .with_id(id)
        .with_position(0., 0., 1.)
        .with_anchor(Anchor::Middle)
        .with_stretch(Stretch::XY {
            x_margin: 0.,
            y_margin: 0.,
            keep_aspect_ratio: false,
        })
        .transparent()
}
