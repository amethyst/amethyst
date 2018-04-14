use super::{Anchor, Anchored, FontAsset, FontHandle, MouseReactive, Stretch, Stretched, TtfFormat, UiImage,
            UiText, UiTransform};
///! A clickable button.
use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::Parent;
use amethyst_renderer::Texture;
use shred::SystemData;
use specs::{Entities, Entity, Fetch, World, WriteStorage};

const DEFAULT_Z: f32 = -1.0;
const DEFAULT_WIDTH: f32 = 128.0;
const DEFAULT_HEIGHT: f32 = 64.0;
const DEFAULT_TAB_ORDER: i32 = 9;
const DEFAULT_BKGD_COLOR: [f32; 4] = [0.82, 0.83, 0.83, 1.0];
const DEFAULT_TXT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const DEFAULT_FONT_NAME: &'static str = "font/square.ttf";

/// Container that wraps the resources we need to initialize button defaults
#[derive(SystemData)]
pub struct UiButtonResources<'a> {
    font_asset: Fetch<'a, AssetStorage<FontAsset>>,
    loader: Fetch<'a, Loader>,
    texture_asset: Fetch<'a, AssetStorage<Texture>>,
}

/// Container for all the resources the builder needs to make a new UiButton.
#[derive(SystemData)]
struct UiButtonBuilderResources<'a> {
    anchored: WriteStorage<'a, Anchored>,
    entities: Entities<'a>,
    image: WriteStorage<'a, UiImage>,
    mouse_reactive: WriteStorage<'a, MouseReactive>,
    parent: WriteStorage<'a, Parent>,
    stretched: WriteStorage<'a, Stretched>,
    text: WriteStorage<'a, UiText>,
    transform: WriteStorage<'a, UiTransform>,
}

impl<'a> UiButtonResources<'a> {
    /// Grab the resources we need from the world.
    pub fn from_world(world: &'a World) -> Self {
        Self::fetch(&world.res, 0)
    }
}

impl<'a> UiButtonBuilderResources<'a> {
    /// Grab the resources we need from the world.
    pub fn from_world(world: &'a World) -> Self {
        Self::fetch(&world.res, 0)
    }
}

/// Builder for a `UiButton`.
pub struct UiButtonBuilder<'a> {
    name: &'a str,
    image: UiImage,
    text: UiText,
    anchored: Option<Anchored>,
    parent: Option<Parent>,
    stretched: Option<Stretched>,
    transform: Option<UiTransform>,
}

/// A clickable button.
pub struct UiButton {
    /// The actual text of the button.
    pub text: Entity,
    /// Represents the background of the image. Defaults to a grey rectangle.
    pub image: Entity,
}

impl<'a> UiButtonBuilder<'a> {
    /// Construct a new UiButtonBuilder.
    /// This allows easy use of default values for text and button appearance and allows the user
    /// to easily set other UI-related options.
    pub fn new<'b, S: ToString>(
        name: &'a str,
        text: S,
        resources: UiButtonResources<'b>,
    ) -> UiButtonBuilder<'a> {
        let (text, image) = {
            let loader = &resources.loader;

            let font = loader.load(
                DEFAULT_FONT_NAME,
                TtfFormat,
                Default::default(),
                (),
                &resources.font_asset,
            );
            let text = UiText::new(font, text.to_string(), DEFAULT_TXT_COLOR, 32.0);
            let grey =
                loader.load_from_data(DEFAULT_BKGD_COLOR.into(), (), &resources.texture_asset);
            let image = UiImage { texture: grey };
            (text, image)
        };

        UiButtonBuilder {
            name,
            image,
            text,
            anchored: None,
            parent: None,
            stretched: None,
            transform: None,
        }
    }

    /// Add an anchor to the button.
    pub fn with_anchored(mut self, anchored: Anchored) -> Self {
        self.anchored = Some(anchored);
        self
    }

    /// Add a parent to the button.
    pub fn with_parent(mut self, parent: Parent) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Stretch the button.
    pub fn with_stretched(mut self, stretched: Stretched) -> Self {
        self.stretched = Some(stretched);
        self
    }

    /// Add a UiTransform to the image to offset it within the UI.
    pub fn with_transform(mut self, transform: UiTransform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// This will completely replace the UiText object representing the button's text.
    /// Use this if you want to change more than just the characters, but the font size, color,
    /// etc. as well.
    /// Use [`with_text`](#with_text) to just change the underlying text.
    pub fn with_uitext(mut self, text: UiText) -> Self {
        self.text = text;
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
        self.text.text = text.to_string();
        self
    }

    /// Replace the default UiImage with `image`.
    pub fn with_image(mut self, image: UiImage) -> Self {
        self.image = image;
        self
    }

    /// Use a different font for the button text.
    pub fn with_font(mut self, font: FontHandle) -> Self {
        self.text.font = font;
        self
    }

    /// Provide an X and Y position for the button.
    ///
    /// This will create a default UiTransform if one is not already attached.
    /// See `DEFAULT_Z`, `DEFAULT_WIDTH`, `DEFAULT_HEIGHT`, and `DEFAULT_TAB_ORDER` for
    /// the values that will be provided to the default UiTransform.
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.transform = if let Some(mut t) = self.transform.take() {
            t.local_x = x;
            t.global_x = x;
            t.local_y = y;
            t.global_y = y;
            Some(t)
        } else {
            let mut id = self.name.to_string();
            id.push_str("_new_transform");
            Some(UiTransform::new(
                id,
                x,
                y,
                DEFAULT_Z,
                DEFAULT_WIDTH,
                DEFAULT_HEIGHT,
                DEFAULT_TAB_ORDER,
            ))
        };
        self
    }

    fn build(mut self, mut res: UiButtonBuilderResources) -> UiButton {
        let image_entity = res.entities.create();
        res.image.insert(image_entity, self.image);
        res.mouse_reactive.insert(image_entity, MouseReactive);
        if let Some(parent) = self.parent.take() {
            res.parent.insert(image_entity, parent);
        }
        if let Some(transform) = self.transform.take() {
            res.transform.insert(image_entity, transform);
        }
        if let Some(anchored) = self.anchored.take() {
            res.anchored.insert(image_entity, anchored);
        }
        if let Some(stretched) = self.stretched.take() {
            res.stretched.insert(image_entity, stretched);
        }

        let mut id = self.name.to_string();
        id.push_str("_btn_txt");
        let text_entity = res.entities.create();
        res.transform
            .insert(text_entity, UiTransform::new(id, 0., 0., -1., 0., 0., 10));
        res.anchored
            .insert(text_entity, Anchored::new(Anchor::Middle));
        res.stretched
            .insert(text_entity, Stretched::new(Stretch::XY, 0., 0.));
        res.text.insert(text_entity, self.text);
        res.parent.insert(
            text_entity,
            Parent {
                entity: image_entity.clone(),
            },
        );

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
