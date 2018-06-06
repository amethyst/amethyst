use super::{Anchor, Anchored, FontAsset, FontHandle, MouseReactive, Stretch, Stretched, TtfFormat,
            UiImage, UiText, UiTransform};
///! A clickable button.
use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::specs::prelude::{Entities, Entity, Read, ReadExpect, World, WriteStorage};
use amethyst_core::Parent;
use amethyst_renderer::{Texture, TextureHandle};
use shred::SystemData;

const DEFAULT_Z: f32 = -1.0;
const DEFAULT_WIDTH: f32 = 128.0;
const DEFAULT_HEIGHT: f32 = 64.0;
const DEFAULT_TAB_ORDER: i32 = 9;
const DEFAULT_BKGD_COLOR: [f32; 4] = [0.82, 0.83, 0.83, 1.0];
const DEFAULT_TXT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const DEFAULT_FONT_NAME: &'static str = "font/square.ttf";

/// Container for all the resources the builder needs to make a new UiButton.
#[derive(SystemData)]
pub struct UiButtonBuilderResources<'a> {
    font_asset: Read<'a, AssetStorage<FontAsset>>,
    texture_asset: Read<'a, AssetStorage<Texture>>,
    loader: ReadExpect<'a, Loader>,
    anchored: WriteStorage<'a, Anchored>,
    entities: Entities<'a>,
    image: WriteStorage<'a, UiImage>,
    mouse_reactive: WriteStorage<'a, MouseReactive>,
    parent: WriteStorage<'a, Parent>,
    stretched: WriteStorage<'a, Stretched>,
    text: WriteStorage<'a, UiText>,
    transform: WriteStorage<'a, UiTransform>,
}

impl<'a> UiButtonBuilderResources<'a> {
    /// Grab the resources we need from the world.
    pub fn from_world(world: &'a World) -> Self {
        Self::fetch(&world.res)
    }
}

pub struct UiButtonBuilder {
    name: String,
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    tab_order: i32,
    anchored: Option<Anchored>,
    stretched: Option<Stretched>,
    text: String,
    text_color: [f32; 4],
    font: Option<FontHandle>,
    font_size: f32,
    image: Option<TextureHandle>,
    parent: Option<Entity>,
}

impl Default for UiButtonBuilder {
    fn default() -> Self {
        UiButtonBuilder {
            name: "".to_string(),
            x: 0.,
            y: 0.,
            z: DEFAULT_Z,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            tab_order: DEFAULT_TAB_ORDER,
            anchored: None,
            stretched: None,
            text: "".to_string(),
            text_color: DEFAULT_TXT_COLOR,
            font: None,
            font_size: 32.,
            image: None,
            parent: None,
        }
    }
}

/// A clickable button.
pub struct UiButton {
    /// The actual text of the button.
    pub text: Entity,
    /// Represents the background of the image. Defaults to a grey rectangle.
    pub image: Entity,
}

impl UiButtonBuilder {
    /// Construct a new UiButtonBuilder.
    /// This allows easy use of default values for text and button appearance and allows the user
    /// to easily set other UI-related options.
    pub fn new<N: ToString, S: ToString>(name: N, text: S) -> UiButtonBuilder {
        let mut builder = UiButtonBuilder::default();
        builder.name = name.to_string();
        builder.text = text.to_string();
        builder
    }

    /// Add an anchor to the button.
    pub fn with_anchored(mut self, anchored: Anchored) -> Self {
        self.anchored = Some(anchored);
        self
    }

    /// Add a parent to the button.
    pub fn with_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Stretch the button.
    pub fn with_stretched(mut self, stretched: Stretched) -> Self {
        self.stretched = Some(stretched);
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

    /// Replace the default UiImage with `image`.
    pub fn with_image(mut self, image: TextureHandle) -> Self {
        self.image = Some(image);
        self
    }

    /// Use a different font for the button text.
    pub fn with_font(mut self, font: FontHandle) -> Self {
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
    pub fn with_tab_order(mut self, tab_order: i32) -> Self {
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

    // unwraps are safe because we create the entities inside
    fn build(mut self, mut res: UiButtonBuilderResources) -> UiButton {
        let mut id = self.name.clone();
        let image_entity = res.entities.create();
        res.transform
            .insert(
                image_entity,
                UiTransform::new(
                    self.name,
                    self.x,
                    self.y,
                    self.z,
                    self.width,
                    self.height,
                    self.tab_order,
                ),
            )
            .unwrap();
        let image_handle = self.image.unwrap_or_else(|| {
            res.loader
                .load_from_data(DEFAULT_BKGD_COLOR.into(), (), &res.texture_asset)
        });

        res.image
            .insert(
                image_entity,
                UiImage {
                    texture: image_handle,
                },
            )
            .unwrap();
        res.mouse_reactive
            .insert(image_entity, MouseReactive)
            .unwrap();
        if let Some(parent) = self.parent.take() {
            res.parent
                .insert(image_entity, Parent { entity: parent })
                .unwrap();
        }
        if let Some(anchored) = self.anchored.take() {
            res.anchored.insert(image_entity, anchored).unwrap();
        }
        if let Some(stretched) = self.stretched.take() {
            res.stretched.insert(image_entity, stretched).unwrap();
        }

        id.push_str("_btn_txt");
        let text_entity = res.entities.create();
        res.transform
            .insert(
                text_entity,
                UiTransform::new(id, 0., 0., -1., 0., 0., 10).as_transparent(),
            )
            .unwrap();
        res.anchored
            .insert(text_entity, Anchored::new(Anchor::Middle))
            .unwrap();
        res.stretched
            .insert(text_entity, Stretched::new(Stretch::XY, 0., 0.))
            .unwrap();
        let font_handle = self.font.unwrap_or_else(|| {
            res.loader.load(
                DEFAULT_FONT_NAME,
                TtfFormat,
                Default::default(),
                (),
                &res.font_asset,
            )
        });
        res.text
            .insert(
                text_entity,
                UiText::new(font_handle, self.text, self.text_color, self.font_size),
            )
            .unwrap();
        res.parent
            .insert(
                text_entity,
                Parent {
                    entity: image_entity.clone(),
                },
            )
            .unwrap();

        UiButton {
            text: text_entity,
            image: image_entity,
        }
    }

    /// Create the UiButton based on provided configuration parameters.
    pub fn build_from_world(self, world: &World) -> UiButton {
        self.build(UiButtonBuilderResources::from_world(world))
    }
}
