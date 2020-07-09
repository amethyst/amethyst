use crate::{
    define_widget, font::default::get_default_font, Anchor, FontAsset, FontHandle, LineMode,
    Stretch, UiText, UiTransform, WidgetId, Widgets,
};

use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::ecs::{
    prelude::{Entities, Entity, Read, ReadExpect, World, WriteExpect, WriteStorage},
    shred::{ResourceId, SystemData},
};

const DEFAULT_Z: f32 = 1.0;
const DEFAULT_WIDTH: f32 = 128.0;
const DEFAULT_HEIGHT: f32 = 64.0;
const DEFAULT_TXT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

define_widget!(UiLabel =>
    entities: [text_entity]
    components: [
        (has UiTransform as position on text_entity),
        (has UiText as text on text_entity)
    ]
);

/// Container for all the resources the builder needs to make a new UiLabel.
#[allow(missing_debug_implementations)]
#[derive(SystemData)]
pub struct UiLabelBuilderResources<'a, I: WidgetId = u32>
where
    I: WidgetId,
{
    font_asset: Read<'a, AssetStorage<FontAsset>>,
    loader: ReadExpect<'a, Loader>,
    entities: Entities<'a>,
    text: WriteStorage<'a, UiText>,
    transform: WriteStorage<'a, UiTransform>,
    label_widgets: WriteExpect<'a, Widgets<UiLabel, I>>,
}

/// Convenience structure for building a label
#[derive(Debug)]
pub struct UiLabelBuilder<I = u32>
where
    I: WidgetId,
{
    id: Option<I>,
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    anchor: Anchor,
    stretch: Stretch,
    text: String,
    text_color: [f32; 4],
    font: Option<FontHandle>,
    font_size: f32,
    line_mode: LineMode,
    align: Anchor,
    parent: Option<Entity>,
}

impl<'a, I> Default for UiLabelBuilder<I>
where
    I: WidgetId + 'static,
{
    fn default() -> Self {
        UiLabelBuilder {
            id: None,
            x: 0.,
            y: 0.,
            z: DEFAULT_Z,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            anchor: Anchor::TopLeft,
            stretch: Stretch::NoStretch,
            text: "".to_string(),
            text_color: DEFAULT_TXT_COLOR,
            font: None,
            font_size: 32.,
            line_mode: LineMode::Single,
            align: Anchor::Middle,
            parent: None,
        }
    }
}

impl<'a, I> UiLabelBuilder<I>
where
    I: WidgetId + 'static,
{
    /// Construct a new UiLabelBuilder.
    /// This allows the user to easily build a UI element with a text that can
    /// easily be retrieved and updated through the appropriate resource,
    /// see [`Widgets`](../struct.Widgets.html).
    pub fn new<S: ToString>(text: S) -> UiLabelBuilder<I> {
        let mut builder = UiLabelBuilder::default();
        builder.text = text.to_string();
        builder
    }

    /// Sets an ID for this widget. The type of this ID will determine which `Widgets`
    /// resource this widget will be added to, see [`Widgets`](../struct.Widgets.html).
    pub fn with_id(mut self, id: I) -> Self {
        self.id = Some(id);
        self
    }

    /// Set button size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

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

    /// Set text color
    pub fn with_text_color(mut self, text_color: [f32; 4]) -> Self {
        self.text_color = text_color;
        self
    }

    /// Use a different font for the button text.
    pub fn with_font(mut self, font: FontHandle) -> Self {
        self.font = Some(font);
        self
    }

    /// Set font size
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
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

    /// Add a parent to the button.
    pub fn with_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Build this with the `UiLabelBuilderResources`.
    pub fn build(self, mut res: UiLabelBuilderResources<'a, I>) -> (I, UiLabel) {
        let text_entity = res.entities.create();
        let widget = UiLabel::new(text_entity);

        let id = {
            let widget = widget.clone();

            if let Some(id) = self.id {
                let added_id = id.clone();
                res.label_widgets.add_with_id(id, widget);
                added_id
            } else {
                res.label_widgets.add(widget)
            }
        };

        res.transform
            .insert(
                text_entity,
                UiTransform::new(
                    format!("{}_label", id),
                    self.anchor,
                    Anchor::Middle,
                    self.x,
                    self.y,
                    self.z,
                    self.width,
                    self.height,
                )
                .with_stretch(self.stretch),
            )
            .expect("Unreachable: Inserting newly created entity");

        let font_handle = self
            .font
            .unwrap_or_else(|| get_default_font(&res.loader, &res.font_asset));

        res.text
            .insert(
                text_entity,
                UiText::new(
                    font_handle,
                    self.text,
                    self.text_color,
                    self.font_size,
                    self.line_mode,
                    self.align,
                ),
            )
            .expect("Unreachable: Inserting newly created entity");

        (id, widget)
    }

    /// Create the UiLabel based on provided configuration parameters.
    pub fn build_from_world(self, world: &World) -> (I, UiLabel) {
        self.build(UiLabelBuilderResources::<I>::fetch(&world))
    }
}
