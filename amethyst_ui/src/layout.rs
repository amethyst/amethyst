use super::UiTransform;
use amethyst_core::specs::prelude::{
    BitSet, InsertedFlag, Join, ModifiedFlag, ReadExpect, ReadStorage, ReaderId, Resources, System,
    WriteStorage,
};
use amethyst_core::{HierarchyEvent, Parent, ParentHierarchy};
use amethyst_renderer::ScreenDimensions;

/// Indicates if the position and margins should be calculated in pixel or
/// relative to their parent size.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum ScaleMode {
    /// Use directly the pixel value.
    Pixel,
    /// Use a proportion (%) of the parent's dimensions (or screen, if there is no parent).
    Percent,
}

/// Indicated where the anchor is, relative to the parent (or to the screen, if there is no parent).
/// Follow a normal english Y,X naming.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum Anchor {
    /// Anchors the entity at the top left of the parent.
    TopLeft,
    /// Anchors the entity at the top middle of the parent.
    TopMiddle,
    /// Anchors the entity at the top right of the parent.
    TopRight,
    /// Anchors the entity at the middle left of the parent.
    MiddleLeft,
    /// Anchors the entity at the center of the parent.
    Middle,
    /// Anchors the entity at the middle right of the parent.
    MiddleRight,
    /// Anchors the entity at the bottom left of the parent.
    BottomLeft,
    /// Anchors the entity at the bottom middle of the parent.
    BottomMiddle,
    /// Anchors the entity at the bottom right of the parent.
    BottomRight,
}

impl Anchor {
    /// Returns the normalized offset using the `Anchor` setting.
    /// The normalized offset is a [-0.5,0.5] value
    /// indicating the relative offset multiplier from the parent's position (centered).
    pub fn norm_offset(&self) -> (f32, f32) {
        match self {
            Anchor::TopLeft => (-0.5, 0.5),
            Anchor::TopMiddle => (0.0, 0.5),
            Anchor::TopRight => (0.5, 0.5),
            Anchor::MiddleLeft => (-0.5, 0.0),
            Anchor::Middle => (0.0, 0.0),
            Anchor::MiddleRight => (0.5, 0.0),
            Anchor::BottomLeft => (-0.5, -0.5),
            Anchor::BottomMiddle => (0.0, -0.5),
            Anchor::BottomRight => (0.5, -0.5),
        }
    }
}

/// Indicates if a component should be stretched.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Stretch {
    /// No stretching occurs
    NoStretch,
    /// Stretches on the X axis.
    X {
        /// The margin length for the width
        x_margin: f32,
    },
    /// Stretches on the Y axis.
    Y {
        /// The margin length for the height
        y_margin: f32,
    },
    /// Stretches on both axes.
    XY {
        /// The margin length for the width
        x_margin: f32,
        /// The margin length for the height
        y_margin: f32,
    },
}

/// Manages the `Parent` component on entities having `UiTransform`
/// It does almost the same as the `TransformSystem`, but with some differences,
/// like `UiTransform` alignment and stretching.
#[derive(Default)]
pub struct UiTransformSystem {
    transform_modified: BitSet,

    inserted_transform_id: Option<ReaderId<InsertedFlag>>,
    modified_transform_id: Option<ReaderId<ModifiedFlag>>,

    parent_events_id: Option<ReaderId<HierarchyEvent>>,

    screen_size: (f32, f32),
}

impl<'a> System<'a> for UiTransformSystem {
    type SystemData = (
        WriteStorage<'a, UiTransform>,
        ReadStorage<'a, Parent>,
        ReadExpect<'a, ScreenDimensions>,
        ReadExpect<'a, ParentHierarchy>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut transforms, parents, screen_dim, hierarchy) = data;
        #[cfg(feature = "profiler")]
        profile_scope!("ui_parent_system");

        self.transform_modified.clear();

        transforms.populate_inserted(
            &mut self.inserted_transform_id.as_mut().unwrap(),
            &mut self.transform_modified,
        );
        transforms.populate_modified(
            &mut self.modified_transform_id.as_mut().unwrap(),
            &mut self.transform_modified,
        );

        for event in hierarchy
            .changed()
            .read(&mut self.parent_events_id.as_mut().unwrap())
        {
            if let HierarchyEvent::Modified(entity) = *event {
                self.transform_modified.add(entity.id());
            }
        }

        let current_screen_size = (screen_dim.width(), screen_dim.height());
        let screen_resized = current_screen_size != self.screen_size;
        self.screen_size = current_screen_size;
        if screen_resized {
            process_root_iter(
                (&mut transforms, !&parents).join().map(|i| i.0),
                &*screen_dim,
            );
        } else {
            process_root_iter(
                (&mut transforms, !&parents, &self.transform_modified)
                    .join()
                    .map(|i| i.0),
                &*screen_dim,
            );
        }

        // Populate the modifications we just did.
        transforms.populate_modified(
            &mut self.modified_transform_id.as_mut().unwrap(),
            &mut self.transform_modified,
        );

        // Compute transforms with parents.
        for entity in hierarchy.all() {
            {
                let self_dirty = self.transform_modified.contains(entity.id());
                let parent_entity = parents.get(*entity).unwrap().entity;
                let parent_dirty = self.transform_modified.contains(parent_entity.id());
                if parent_dirty || self_dirty || screen_resized {
                    let parent_transform_copy = transforms.get(parent_entity).cloned();
                    let transform = transforms.get_mut(*entity);
                    if parent_transform_copy.is_none() || transform.is_none() {
                        continue;
                    }
                    let parent_transform_copy = parent_transform_copy.unwrap();
                    let mut transform = transform.unwrap();
                    let norm = transform.anchor.norm_offset();
                    transform.pixel_x =
                        parent_transform_copy.pixel_x + parent_transform_copy.pixel_width * norm.0;
                    transform.pixel_y =
                        parent_transform_copy.pixel_y + parent_transform_copy.pixel_height * norm.1;
                    transform.global_z = parent_transform_copy.global_z + transform.local_z;

                    let new_size = match transform.stretch {
                        Stretch::NoStretch => (transform.width, transform.height),
                        Stretch::X { x_margin } => (
                            parent_transform_copy.pixel_width - x_margin * 2.0,
                            transform.height,
                        ),
                        Stretch::Y { y_margin } => (
                            transform.width,
                            parent_transform_copy.pixel_height - y_margin * 2.0,
                        ),
                        Stretch::XY { x_margin, y_margin } => (
                            parent_transform_copy.pixel_width - x_margin * 2.0,
                            parent_transform_copy.pixel_height - y_margin * 2.0,
                        ),
                    };
                    transform.width = new_size.0;
                    transform.height = new_size.1;
                    match transform.scale_mode {
                        ScaleMode::Pixel => {
                            transform.pixel_x += transform.local_x;
                            transform.pixel_y += transform.local_y;
                            transform.pixel_width = transform.width;
                            transform.pixel_height = transform.height;
                        }
                        ScaleMode::Percent => {
                            transform.pixel_x +=
                                transform.local_x * parent_transform_copy.pixel_width;
                            transform.pixel_y +=
                                transform.local_y * parent_transform_copy.pixel_height;
                            transform.pixel_width =
                                transform.width * parent_transform_copy.pixel_width;
                            transform.pixel_height =
                                transform.height * parent_transform_copy.pixel_height;
                        }
                    }
                }
            }
            // Populate the modifications we just did.
            transforms.populate_modified(
                &mut self.modified_transform_id.as_mut().unwrap(),
                &mut self.transform_modified,
            );
        }
        // We need to treat any changes done inside the system as non-modifications, so we read out
        // any events that were generated during the system run
        transforms.populate_inserted(
            &mut self.inserted_transform_id.as_mut().unwrap(),
            &mut self.transform_modified,
        );
        transforms.populate_modified(
            &mut self.modified_transform_id.as_mut().unwrap(),
            &mut self.transform_modified,
        );
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.parent_events_id = Some(res.fetch_mut::<ParentHierarchy>().track());
        let mut transforms = WriteStorage::<UiTransform>::fetch(res);
        self.inserted_transform_id = Some(transforms.track_inserted());
        self.modified_transform_id = Some(transforms.track_modified());
    }
}

fn process_root_iter<'a, I>(iter: I, screen_dim: &ScreenDimensions)
where
    I: Iterator<Item = &'a mut UiTransform>,
{
    for transform in iter {
        let norm = transform.anchor.norm_offset();
        transform.pixel_x = screen_dim.width() / 2.0 + screen_dim.width() * norm.0;
        transform.pixel_y = screen_dim.height() / 2.0 + screen_dim.height() * norm.1;
        transform.global_z = transform.local_z;

        let new_size = match transform.stretch {
            Stretch::NoStretch => (transform.width, transform.height),
            Stretch::X { x_margin } => {
                (screen_dim.width() - x_margin * 2.0, transform.height)
            }
            Stretch::Y { y_margin } => {
                (transform.width, screen_dim.height() - y_margin * 2.0)
            }
            Stretch::XY { x_margin, y_margin } => (
                screen_dim.width() - x_margin * 2.0,
                screen_dim.height() - y_margin * 2.0,
            ),
        };
        transform.width = new_size.0;
        transform.height = new_size.1;
        match transform.scale_mode {
            ScaleMode::Pixel => {
                transform.pixel_x += transform.local_x;
                transform.pixel_y += transform.local_y;
                transform.pixel_width = transform.width;
                transform.pixel_height = transform.height;
            }
            ScaleMode::Percent => {
                transform.pixel_x += transform.local_x * screen_dim.width();
                transform.pixel_y += transform.local_y * screen_dim.height();
                transform.pixel_width = transform.width * screen_dim.width();
                transform.pixel_height = transform.height * screen_dim.height();
            }
        }
    }
}
