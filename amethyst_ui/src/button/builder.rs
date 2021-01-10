use std::marker::PhantomData;

use amethyst_assets::{DefaultLoader, Handle, Loader, ProcessingQueue};
use amethyst_audio::SourceHandle;
use amethyst_core::{
    ecs::*,
    transform::{Children, Parent, Transform},
};
use amethyst_rendy::{
    palette::Srgba, rendy::texture::palette::load_from_srgba, types::TextureData,
};
use smallvec::{smallvec, SmallVec};

use crate::{
    Anchor, FontAsset, Interactable, LineMode, Selectable, Stretch, UiButton, UiButtonAction,
    UiButtonActionRetrigger,
    UiButtonActionType::{self, *},
    UiImage, UiPlaySoundAction, UiSoundRetrigger, UiText, UiTransform, WidgetId, Widgets,
};

const DEFAULT_Z: f32 = 1.0;
const DEFAULT_WIDTH: f32 = 128.0;
const DEFAULT_HEIGHT: f32 = 64.0;
const DEFAULT_TAB_ORDER: u32 = 9;
const DEFAULT_BKGD_COLOR: [f32; 4] = [0.82, 0.83, 0.83, 1.0];
const DEFAULT_TXT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Convenience structure for building a button
/// Note that since there can only be one "ui_loader" in use, and WidgetId of the UiBundle and
/// UiButtonBuilder should match, you can only use one type of WidgetId, e.g. you cant use both
/// UiButtonBuilder<(), u32> and UiButtonBuilder<(), String>.
#[derive(Debug, Clone)]
pub struct UiButtonBuilder<G, I: WidgetId> {
    id: Option<I>,
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    tab_order: u32,
    anchor: Anchor,
    stretch: Stretch,
    text: String,
    text_color: [f32; 4],
    font: Option<Handle<FontAsset>>,
    font_size: f32,
    line_mode: LineMode,
    align: Anchor,
    image: Option<UiImage>,
    parent: Option<Entity>,
    on_click_start_sound: Option<UiPlaySoundAction>,
    on_click_stop_sound: Option<UiPlaySoundAction>,
    on_hover_sound: Option<UiPlaySoundAction>,
    // SetTextColor and SetImage can occur on click/hover start,
    // Unset for both on click/hover stop, so we only need 2 max.
    on_click_start: SmallVec<[UiButtonActionType; 2]>,
    on_click_stop: SmallVec<[UiButtonActionType; 2]>,
    on_hover_start: SmallVec<[UiButtonActionType; 2]>,
    on_hover_stop: SmallVec<[UiButtonActionType; 2]>,
    _phantom: PhantomData<G>,
}

impl<G, I> Default for UiButtonBuilder<G, I>
where
    I: WidgetId,
{
    fn default() -> Self {
        UiButtonBuilder {
            id: None,
            x: 0.,
            y: 0.,
            z: DEFAULT_Z,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            tab_order: DEFAULT_TAB_ORDER,
            anchor: Anchor::TopLeft,
            stretch: Stretch::NoStretch,
            text: "".to_string(),
            text_color: DEFAULT_TXT_COLOR,
            font: None,
            font_size: 32.,
            line_mode: LineMode::Single,
            align: Anchor::Middle,
            image: None,
            parent: None,
            on_click_start_sound: None,
            on_click_stop_sound: None,
            on_hover_sound: None,
            on_click_start: smallvec![],
            on_click_stop: smallvec![],
            on_hover_start: smallvec![],
            on_hover_stop: smallvec![],
            _phantom: PhantomData,
        }
    }
}

impl<'a, G: PartialEq + Send + Sync + 'static, I: WidgetId> UiButtonBuilder<G, I> {
    /// Construct a new UiButtonBuilder.
    /// This allows easy use of default values for text and button appearance and allows the user
    /// to easily set other UI-related options. It also allows easy retrieval and updating through
    /// the appropriate widgets resouce, see [`Widgets`](../../struct.Widgets.html).
    pub fn new<S: ToString>(text: S) -> UiButtonBuilder<G, I> {
        UiButtonBuilder {
            text: text.to_string(),
            ..Default::default()
        }
    }

    /// Sets an ID for this widget. The type of this ID will determine which `Widgets`
    /// resource this widget will be added to, see see [`Widgets`](../../struct.Widgets.html).
    pub fn with_id(mut self, id: I) -> Self {
        self.id = Some(id);
        self
    }

    /// Add a parent to the button.
    pub fn with_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Add an anchor to the button.
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Stretch the button.
    pub fn with_stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = stretch;
        self
    }

    /// This will set the rendered characters within the button. Use this to just change what
    /// characters will appear. If you need to change the font size, color, etc., then you should
    /// use
    /// [`with_uitext`](#with_uitext) and provide a new `UiText` object.
    pub fn with_text<S>(mut self, text: S) -> Self
    where
        S: ToString,
    {
        self.text = text.to_string();
        self
    }

    /// Replace the default Handle<Texture> with `image`.
    pub fn with_image(mut self, image: UiImage) -> Self {
        self.image = Some(image);
        self
    }

    /// Use a different font for the button text.
    pub fn with_font(mut self, font: Handle<FontAsset>) -> Self {
        self.font = Some(font);
        self
    }

    /// Provide an X and Y position for the button.
    ///
    /// This will create a default UiTransform if one is not already attached.
    /// See `DEFAULT_Z`, `DEFAULT_WIDTH`, `DEFAULT_HEIGHT`, and `DEFAULT_TAB_ORDER` for
    /// the values that will be provided to the default UiTransform.
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Provide a Z position, i.e UI layer
    pub fn with_layer(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Set button size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set button tab order
    pub fn with_tab_order(mut self, tab_order: u32) -> Self {
        self.tab_order = tab_order;
        self
    }

    /// Set font size
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set text color
    pub fn with_text_color(mut self, text_color: [f32; 4]) -> Self {
        self.text_color = text_color;
        self
    }

    /// Set text line mode
    pub fn with_line_mode(mut self, line_mode: LineMode) -> Self {
        self.line_mode = line_mode;
        self
    }

    /// Set text align
    pub fn with_align(mut self, align: Anchor) -> Self {
        self.align = align;
        self
    }

    /// Text color to use when the mouse is hovering over this button
    pub fn with_hover_text_color(mut self, text_color: [f32; 4]) -> Self {
        self.on_hover_start.push(SetTextColor(text_color));
        self.on_hover_stop.push(UnsetTextColor(text_color));
        self
    }

    /// Set text color when the button is pressed
    pub fn with_press_text_color(mut self, text_color: [f32; 4]) -> Self {
        self.on_click_start.push(SetTextColor(text_color));
        self.on_click_stop.push(UnsetTextColor(text_color));
        self
    }

    /// Button image to use when the mouse is hovering over this button
    pub fn with_hover_image(mut self, image: UiImage) -> Self {
        self.on_hover_start.push(SetImage(image.clone()));
        self.on_hover_stop.push(UnsetTexture(image));
        self
    }

    /// Button image to use when this button is pressed
    pub fn with_press_image(mut self, image: UiImage) -> Self {
        self.on_click_start.push(SetImage(image.clone()));
        self.on_click_stop.push(UnsetTexture(image));
        self
    }

    /// Sound emitted when this button is hovered over
    pub fn with_hover_sound(mut self, sound: SourceHandle) -> Self {
        self.on_hover_sound = Some(UiPlaySoundAction(sound));
        self
    }

    /// Sound emitted when this button is pressed
    pub fn with_press_sound(mut self, sound: SourceHandle) -> Self {
        self.on_click_start_sound = Some(UiPlaySoundAction(sound));
        self
    }

    /// Sound emitted when this button is released
    pub fn with_release_sound(mut self, sound: SourceHandle) -> Self {
        self.on_click_stop_sound = Some(UiPlaySoundAction(sound));
        self
    }

    /// Build this with the `UiButtonBuilderResources`.
    pub fn build_from_world_and_resources(
        mut self,
        world: &mut World,
        resources: &mut Resources,
    ) -> (I, UiButton) {
        let entities = world.extend(vec![(), ()]);

        let (image_entity, text_entity) = (entities[0], entities[1]);

        let widget = UiButton::new(text_entity, image_entity);

        let id = {
            let widget = widget.clone();

            if !resources.contains::<Widgets<UiButton, I>>() {
                resources.insert(Widgets::<UiButton, I>::new());
            }

            let mut button_widgets = resources.get_mut::<Widgets<UiButton, I>>().unwrap();
            if let Some(id) = self.id {
                let added_id = id.clone();
                button_widgets.add_with_id(id, widget);
                added_id
            } else {
                button_widgets.add(widget)
            }
        };

        if !self.on_click_start.is_empty()
            || !self.on_click_stop.is_empty()
            || !self.on_hover_start.is_empty()
            || !self.on_hover_stop.is_empty()
        {
            let button_action_retrigger = UiButtonActionRetrigger {
                on_click_start: actions_with_target(
                    &mut self.on_click_start.into_iter(),
                    image_entity,
                ),
                on_click_stop: actions_with_target(
                    &mut self.on_click_stop.into_iter(),
                    image_entity,
                ),
                on_hover_start: actions_with_target(
                    &mut self.on_hover_start.into_iter(),
                    image_entity,
                ),
                on_hover_stop: actions_with_target(
                    &mut self.on_hover_stop.into_iter(),
                    image_entity,
                ),
            };

            world
                .entry(image_entity)
                .expect("Unreachable: Inserting newly created entity")
                .add_component(button_action_retrigger);
        }

        if self.on_click_start_sound.is_some()
            || self.on_click_stop_sound.is_some()
            || self.on_hover_sound.is_some()
        {
            let sound_retrigger = UiSoundRetrigger {
                on_click_start: self.on_click_start_sound,
                on_click_stop: self.on_click_stop_sound,
                on_hover_start: self.on_hover_sound,
                on_hover_stop: None,
            };

            world
                .entry(image_entity)
                .expect("Unreachable: Inserting newly created entity")
                .add_component(sound_retrigger);
        }

        world
            .entry(image_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(
                UiTransform::new(
                    format!("{}_btn", id),
                    self.anchor,
                    Anchor::Middle,
                    self.x,
                    self.y,
                    self.z,
                    self.width,
                    self.height,
                )
                .with_stretch(self.stretch),
            );

        world
            .entry(image_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(Selectable::<G>::new(self.tab_order));

        let asset_storage = resources.get::<ProcessingQueue<TextureData>>().unwrap();
        let loader = resources
            .get::<DefaultLoader>()
            .expect("Could not get Loader resource");
        let image = self.image.unwrap_or_else(|| {
            UiImage::Texture(
                loader.load_from_data(
                    load_from_srgba(Srgba::new(
                        DEFAULT_BKGD_COLOR[0],
                        DEFAULT_BKGD_COLOR[1],
                        DEFAULT_BKGD_COLOR[2],
                        DEFAULT_BKGD_COLOR[3],
                    ))
                    .into(),
                    (),
                    &asset_storage,
                ),
            )
        });

        world
            .entry(image_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(image);
        world
            .entry(image_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(Interactable);

        if let Some(parent) = self.parent.take() {
            world
                .entry(image_entity)
                .expect("Unreachable: Inserting newly created entity")
                .add_component(Parent(parent));
        }

        world
            .entry(text_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(
                UiTransform::new(
                    format!("{}_btn_text", id),
                    Anchor::Middle,
                    Anchor::Middle,
                    0.,
                    0.,
                    0.01,
                    0.,
                    0.,
                )
                .into_transparent()
                .with_stretch(Stretch::XY {
                    x_margin: 0.,
                    y_margin: 0.,
                    keep_aspect_ratio: false,
                }),
            );
        world
            .entry(text_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(UiText::new(
                self.font,
                self.text,
                self.text_color,
                self.font_size,
                self.line_mode,
                self.align,
            ));
        world
            .entry(image_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(Children(smallvec![text_entity]));

        world
            .entry(text_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(Parent(image_entity));

        //FIXME : The current parent update system in amethyst_core is updating based on the Transform component...
        // That's actually a 'bad' linkage. Later to legion port, we'll replace the system by legion_transform which is better,
        // the following 4 lines won't be usefull anymore.
        world
            .entry(image_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(Transform::default());

        world
            .entry(text_entity)
            .expect("Unreachable: Inserting newly created entity")
            .add_component(Transform::default());

        (id, widget)
    }
}

fn actions_with_target<I>(actions: I, target: Entity) -> Vec<UiButtonAction>
where
    I: Iterator<Item = UiButtonActionType>,
{
    actions
        .map(|action| {
            UiButtonAction {
                target,
                event_type: action,
            }
        })
        .collect()
}
