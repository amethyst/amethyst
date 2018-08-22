use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::prelude::{
    Component, DenseVecStorage, Entity, NullStorage, WriteStorage,
};
use std::f32::INFINITY;

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Default)]
pub struct FlyControlTag;

impl Component for FlyControlTag {
    type Storage = NullStorage<FlyControlTag>;
}

/// Add this to a camera if you want it to be an XY camera.
/// You need to add the XYCameraSystem for it to work.
pub struct XYControlTag {
    /// Speed of the camera on the X axis.
    pub x_speed: f32,
    /// Speed of the camera on the Y axis.
    pub y_speed: f32,
    /// Speed of the camera while zooming.
    pub zoom_speed: f32,
    /// Borders of the local camera position on the X axis.
    pub horizontal_borders: (f32, f32),
    /// Borders of the local camera position on the Y axis.
    pub vertical_borders: (f32, f32),
    /// Borders of the local camera zoom.
    pub zoom_borders: (f32, f32),
}

impl XYControlTag {
    pub fn new(x_speed: f32, y_speed: f32, zoom_speed: f32) -> Self {
        Self {
            x_speed,
            y_speed,
            zoom_speed,
            ..Self::default()
        }
    }

    pub fn with_horizontal_borders(mut self, horizontal_borders: (f32, f32)) -> Self {
        self.horizontal_borders = horizontal_borders;
        self
    }

    pub fn with_vertical_borders(mut self, vertical_borders: (f32, f32)) -> Self {
        self.vertical_borders = vertical_borders;
        self
    }

    pub fn with_zoom_borders(mut self, zoom_borders: (f32, f32)) -> Self {
        self.zoom_borders = zoom_borders;
        self
    }
}

impl Default for XYControlTag {
    fn default() -> Self {
        Self {
            x_speed: 1.0,
            y_speed: 1.0,
            zoom_speed: 1.0,
            horizontal_borders: (-INFINITY, INFINITY),
            vertical_borders: (-INFINITY, INFINITY),
            zoom_borders: (0.0, INFINITY),
        }
    }
}

impl Component for XYControlTag {
    type Storage = DenseVecStorage<XYControlTag>;
}

/// To add an arc ball behaviour, add this to a camera which already has the FlyControlTag added.
#[derive(Debug, Clone)]
pub struct ArcBallControlTag {
    pub target: Entity,
    pub distance: f32,
}

impl Component for ArcBallControlTag {
    type Storage = DenseVecStorage<ArcBallControlTag>;
}

/// `PrefabData` for loading control tags on an `Entity`
///
/// Will always load a `FlyControlTag`
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ControlTagPrefab {
    /// Place `ArcBallControlTag` on the `Entity`
    pub arc_ball: Option<(usize, f32)>,
}

impl<'a> PrefabData<'a> for ControlTagPrefab {
    type SystemData = (
        WriteStorage<'a, FlyControlTag>,
        WriteStorage<'a, ArcBallControlTag>,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        system_data.0.insert(entity, FlyControlTag)?;
        if let Some((index, distance)) = self.arc_ball {
            system_data.1.insert(
                entity,
                ArcBallControlTag {
                    target: entities[index],
                    distance,
                },
            )?;
        }
        Ok(())
    }
}
