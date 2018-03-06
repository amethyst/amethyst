//! Clickable button.
use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::Parent;
use amethyst_renderer::ScreenDimensions;
use hibitset::BitSet;
use specs::{Component, Entities, Entity, Fetch, FlaggedStorage, Join, ReadStorage, System, VecStorage, World,
            WriteStorage};

use super::{FontAsset, TtfFormat, UiImage, UiText, Stretched, Stretch, MouseReactive, UiEvent, UiFocused, UiTransform};

/// A clickable button.
/// It has an X, Y, Z coordinate as well as an associated text and 
/// an image. The default button is black text on a grey background.
pub struct UiButton
{
    pub text: UiText,
    pub image: UiImage,
}

/// `UiButtonBuilder` is an interface that allows for the creation of a
/// [`UiButton`](struct.UiButton.html) using a custom set of configuration.
pub struct UiButtonBuilder {
    loader: Loader,
    world: World,
    text: Option<UiText>,
    image: Option<UiImage>,
    x: f32,
    y: f32,
    z: f32,
}


impl UiButtonBuilder {
    /// Creates a new [UiButtonBuilder](struct.UiButtonBuilder.html) instance
    /// This is a more verbose way of initializing a button if you need something other than the
    /// defaults
    /// # Returns 
    ///
    /// Returns a `Result` type wrapping the `UiButton` type. See
    /// [errors](struct.UiButton.html#errors) for a full list of possible
    /// errors that can happen in the creation of a UiButton object.
    ///
    /// # Parameters
    /// - `loader`: [`Loader`](struct.Loader.html) used to generate
    ///             default values for the button
    /// # Errors
    ///
    /// UiButton will return an error if the 
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ui::UiButton;
    ///
    ///
    pub fn new(loader: Loader, world: World) -> Self {
        UiButtonBuilder {
            loader: loader,
            world: world,
            text: None,
            image: None,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn with_text(mut self, text: &UiText) -> Self {
        self.text = Some(text.clone());
        self
    }
    
    /// Create a button with the text string `label`. Use the default or
    /// previously configured font, font color, and font size 
    pub fn with_label<T: ToString>(mut self, label: T) -> Self {
        let font = self.loader.load(
            "font/square.ttf",
            TtfFormat,
            Default::default(),
            (),
            &self.world.read_resource::<AssetStorage<FontAsset>>(),
        );
        self.text = Some(UiText::new(
                font, 
                label.to_string(),  
                [0.82, 0.83, 0.83, 1.0], 
                16.0)
            );
        self
    }

    pub fn with_image(mut self, image: &UiImage) -> Self {
        self.image = Some(image.clone());
        self
    }
}
