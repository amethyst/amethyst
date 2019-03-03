use amethyst_core::{
    nalgebra::{Isometry3, Vector3},
    specs::{Component, NullStorage},
};
/// Indicates that the entity on which this is placed should use the coordinates
/// of the window instead of the coordinates relative to the `Camera`.
/// Mostly used with user interface elements.
/// Do not use with entities located outside of -1000 <= z < 1000.
#[derive(Default, Debug, Clone, Copy)]
pub struct ScreenSpace;

impl Component for ScreenSpace {
    type Storage = NullStorage<Self>;
}

/// The ScreenSpace projection settings.
/// Only entities with a z value between -max_depth/2 + 0.1 and max_depth/2 will be visible.
/// The default max_depth value is 2000.
#[derive(Debug, Clone)]
pub struct ScreenSpaceSettings {
    pub(crate) max_depth: f32,
    pub(crate) view_matrix: [[f32; 4]; 4],
}

impl ScreenSpaceSettings {
    /// Creates a new ScreenSpaceSettings with the specified maximal visible depth.
    pub fn new(max_depth: f32) -> Self {
        let translation = Vector3::<f32>::new(0.0, 0.0, max_depth / 2.0);
        let iso = Isometry3::new(translation, amethyst_core::nalgebra::zero());
        let pos: [[f32; 4]; 4] = iso.inverse().to_homogeneous().into();
        ScreenSpaceSettings {
            max_depth,
            view_matrix: pos,
        }
    }
}

impl Default for ScreenSpaceSettings {
    fn default() -> Self {
        ScreenSpaceSettings::new(2000.0)
    }
}
