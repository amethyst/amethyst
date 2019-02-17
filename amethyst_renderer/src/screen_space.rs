use amethyst_core::specs::{Component, NullStorage};

/// Indicates that the entity on which this is placed should use the coordinates of the window instead of the coordinates relative to the `Camera`.
/// Mostly used with user interface elements.
/// Do not use with entities located outside of -1000 <= z < 1000.
#[derive(Default, Debug, Clone, Copy)]
pub struct ScreenSpace;

impl Component for ScreenSpace {
    type Storage = NullStorage<Self>;
}