use super::{Anchor, Anchored, FontAsset, Stretch, Stretched, TtfFormat, UiImage, UiText,
            UiTransform};
///! A clickable button.
use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::Parent;
use amethyst_renderer::Texture;
use specs::{Entity, World};

use std::marker;

/// Container that wraps the resources we need to initialize button defaults
pub struct UiButtonResources<'a> {
    loader: &'a Loader,
    font_asset: &'a AssetStorage<FontAsset>,
    texture_asset: &'a AssetStorage<Texture>,
}

impl <'a> UiButtonResources<'a> {
    pub fn from_world(world: &World) -> Self {
        UiButtonResources {
            loader: &world.read_resource::<Loader>(),
            font_asset: &world.read_resource::<AssetStorage<FontAsset>>(),
            texture_asset: &world.read_resource::<AssetStorage<Texture>>(),
        }
    }
}

/// Builder for a `UiButton`.
pub struct UiButtonBuilder<'a, 'b> {
    name: &'a str,
    world: &'b mut World,
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

impl<'a, 'b> UiButtonBuilder<'a, 'b> {
    /// Construct a new UiButtonBuilder.
    /// This allows easy use of default values for text and button appearance and allows the user
    /// to easily set other UI-related options.
    pub fn new<S: ToString>(name: &'a str, text: S, world: &'b mut World) -> Self {
        let (text, image) = {
            let loader = world.read_resource::<Loader>();

            let font = loader.load(
                "font/square.ttf",
                TtfFormat,
                Default::default(),
                (),
                &world.read_resource::<AssetStorage<FontAsset>>(),
            );
            let text = UiText::new(font, text.to_string(), [0.0, 0.0, 0.0, 1.0], 32.0);
            let grey = loader.load_from_data(
                [0.82, 0.83, 0.83, 1.0].into(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            );
            let image = UiImage { texture: grey };
            (text, image)
        };
        UiButtonBuilder {
            name: name,
            world: world,
            image: image,
            text: text,
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

    fn build_text(&mut self, image: &Entity) -> Entity {
        let mut id = self.name.to_string();
        id.push_str("_btn_txt");
        self.world
            .create_entity()
            .with(UiTransform::new(id, 0., 0., -1., 0., 0., 10))
            .with(Anchored::new(Anchor::Middle))
            .with(Stretched::new(Stretch::XY, 0., 0.))
            .with(self.text.clone())
            .with(Parent {
                entity: image.clone(),
            })
            .build()
    }

    fn build_image(&mut self) -> Entity {
        let mut image_builder = self.world.create_entity().with(self.image.clone());
        if let Some(parent) = self.parent.take() {
            image_builder = image_builder.with(parent);
        }
        if let Some(transform) = self.transform.take() {
            image_builder = image_builder.with(transform);
        }
        if let Some(anchored) = self.anchored.take() {
            image_builder = image_builder.with(anchored);
        }
        if let Some(stretched) = self.stretched.take() {
            image_builder = image_builder.with(stretched);
        }

        image_builder.build()
    }

    /// Create the UiButton based on provided configuration parameters.
    pub fn build(mut self) -> UiButton {
        let image_entity = self.build_image();
        let text_entity = self.build_text(&image_entity);
        UiButton {
            text: text_entity,
            image: image_entity,
        }
    }
}
