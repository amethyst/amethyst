use super::UiTransform;
use amethyst_core::specs::prelude::{BitSet, Component, Entities, FlaggedStorage, InsertedFlag,
                                    Join, ModifiedFlag, ReadExpect, ReadStorage, ReaderId,
                                    Resources, System, VecStorage, WriteStorage};
use amethyst_core::{HierarchyEvent, Parent, ParentHierarchy};
use amethyst_renderer::ScreenDimensions;

/// Unused, will be implemented in a future PR.
/// Indicated if the position and margins should be calculated in pixel or
/// relative to their parent size.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ScaleMode {
    /// Use directly the pixel value.
    Pixel,
    /// Use a proportion (%) of the parent's dimensions (or screen, if there is no parent).
    Percent,
}

/// Indicated where the anchor is, relative to the parent (or to the screen, if there is no parent).
/// Follow a normal english Y,X naming.
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Indicates if a component should be stretched.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Stretch {
    /// Stretches on the X axis.
    X,
    /// Stretches on the Y axis.
    Y,
    /// Stretches on both axes.
    XY,
}

/// Component indicating that the position of this entity should be relative to the parent's position.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Anchored {
    /// The `Anchor`
    anchor: Anchor,
    /// Defaults to none.
    /// While the position value in UiTransform will be changed,
    /// this keeps track of the offset from the anchor.
    /// By default, it will automatically be set to the UiTransform position before it gets moved by the layout system.
    #[serde(skip)]
    offset: Option<(f32, f32)>,
}

impl Anchored {
    /// Creates a new `Anchored` component using the `Anchor` setting.
    pub fn new(anchor: Anchor) -> Self {
        Anchored {
            anchor,
            offset: None,
        }
    }

    /// Returns the normalized offset using the `Anchor` setting.
    /// The normalized offset is a [-0.5,0.5] value
    /// indicating the relative offset from the parent's position (centered).
    pub fn norm_offset(&self) -> (f32, f32) {
        match self.anchor {
            Anchor::TopLeft => (-0.5, -0.5),
            Anchor::TopMiddle => (0.0, -0.5),
            Anchor::TopRight => (0.5, -0.5),
            Anchor::MiddleLeft => (-0.5, 0.0),
            Anchor::Middle => (0.0, 0.0),
            Anchor::MiddleRight => (0.5, 0.0),
            Anchor::BottomLeft => (-0.5, 0.5),
            Anchor::BottomMiddle => (0.0, 0.5),
            Anchor::BottomRight => (0.5, 0.5),
        }
    }
}

impl Component for Anchored {
    type Storage = VecStorage<Self>;
}

/// Component indicating that an entity should be stretched to fit the parent size
/// on one or multiple axes.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stretched {
    /// The `Stretch` setting.
    /// Indicates on which axes this entity should be stretched.
    stretch: Stretch,
    /// Defaults to 0,0.
    /// Use .with_margin(x,y) to change.
    margin: (f32, f32),
}

impl Stretched {
    /// Create a new `Stretched` component using the `Stretch` setting.
    pub fn new(stretch: Stretch, margin_x: f32, margin_y: f32) -> Self {
        Stretched {
            stretch,
            margin: (margin_x, margin_y),
        }
    }
}

impl Component for Stretched {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

/// Used to initialize the `UiTransform` and `Anchored` offsets when using the `Anchored` component.
pub struct UiLayoutSystem {
    screen_size: (f32, f32),
}

impl UiLayoutSystem {
    /// Creates a new UiLayoutSystem.
    pub fn new() -> Self {
        UiLayoutSystem {
            screen_size: (0.0, 0.0),
        }
    }
}

impl<'a> System<'a> for UiLayoutSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, Anchored>,
        ReadStorage<'a, Parent>,
        ReadExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (entities, mut transform, mut anchor, parent, screen_dim): Self::SystemData) {
        let cur_size = (screen_dim.width(), screen_dim.height());
        let offset_override = self.screen_size != cur_size;
        self.screen_size = cur_size;
        for (entity, mut tr, mut anchor) in (&*entities, &mut transform, &mut anchor).join() {
            if anchor.offset.is_none() || (offset_override && anchor.offset.is_some()) {
                if offset_override && anchor.offset.is_some() {
                    tr.local_x = anchor.offset.unwrap().0;
                    tr.local_y = anchor.offset.unwrap().1;
                }

                anchor.offset = Some((tr.local_x, tr.local_y));

                let norm_offset = anchor.norm_offset();

                // Percent will be implemented in a future PR
                let user_offset = match tr.scale_mode {
                    ScaleMode::Pixel => anchor.offset.unwrap(),
                    ScaleMode::Percent => anchor.offset.unwrap(),
                };

                let middle = (screen_dim.width() / 2.0, screen_dim.height() / 2.0);

                let new_pos_x = middle.0 + norm_offset.0 * screen_dim.width() + user_offset.0;
                let new_pos_y = middle.1 + norm_offset.1 * screen_dim.height() + user_offset.1;
                tr.local_x = new_pos_x;
                tr.local_y = new_pos_y;
                if !parent.contains(entity) {
                    tr.global_x = tr.local_x;
                    tr.global_y = tr.local_y;
                    tr.global_z = tr.local_z;
                }
            }
        }
    }
}

/// Manages the `Parent` component on entities having `UiTransform`
/// It does almost the same as the `TransformSystem`, but with some differences,
/// like `UiTransform` alignment and stretching.
#[derive(Default)]
pub struct UiParentSystem {
    local_modified: BitSet,

    inserted_local_id: Option<ReaderId<InsertedFlag>>,
    modified_local_id: Option<ReaderId<ModifiedFlag>>,

    inserted_stretch_id: Option<ReaderId<InsertedFlag>>,
    modified_stretch_id: Option<ReaderId<ModifiedFlag>>,

    parent_events_id: Option<ReaderId<HierarchyEvent>>,
}

impl<'a> System<'a> for UiParentSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiTransform>,
        ReadStorage<'a, Parent>,
        ReadStorage<'a, Anchored>,
        ReadStorage<'a, Stretched>,
        ReadExpect<'a, ScreenDimensions>,
        ReadExpect<'a, ParentHierarchy>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut locals, parents, anchors, stretches, screen_dim, hierarchy) = data;
        #[cfg(feature = "profiler")]
        profile_scope!("ui_parent_system");

        self.local_modified.clear();

        locals.populate_inserted(
            &mut self.inserted_local_id.as_mut().unwrap(),
            &mut self.local_modified,
        );
        locals.populate_modified(
            &mut self.modified_local_id.as_mut().unwrap(),
            &mut self.local_modified,
        );

        stretches.populate_inserted(
            &mut self.inserted_stretch_id.as_mut().unwrap(),
            &mut self.local_modified,
        );
        stretches.populate_modified(
            &mut self.modified_stretch_id.as_mut().unwrap(),
            &mut self.local_modified,
        );

        for event in hierarchy
            .changed()
            .read(&mut self.parent_events_id.as_mut().unwrap())
        {
            if let HierarchyEvent::Modified(entity) = *event {
                self.local_modified.add(entity.id());
            }
        }

        // Compute transforms with parents.
        for entity in hierarchy.all() {
            let self_dirty = self.local_modified.contains(entity.id());
            let mut combined_transform: Option<(f32, f32, f32)> = None;
            let mut new_size: Option<(f32, f32)> = None;

            match (parents.get(*entity), locals.get(*entity)) {
                (Some(parent), Some(local)) => {
                    let parent_dirty = self.local_modified.contains(parent.entity.id());
                    if parent_dirty || self_dirty {
                        if let Some(parent_global) = locals.get(parent.entity) {
                            combined_transform = Some(match anchors.get(*entity) {
                                Some(anchor) => {
                                    let norm = anchor.norm_offset();
                                    (
                                        parent_global.global_x + parent_global.width * norm.0
                                            + anchor.offset.unwrap().0,
                                        parent_global.global_y + parent_global.height * norm.1
                                            + anchor.offset.unwrap().1,
                                        parent_global.global_z + local.local_z,
                                    )
                                }
                                None => (
                                    parent_global.global_x + local.local_x,
                                    parent_global.global_y + local.local_y,
                                    parent_global.global_z + local.local_z,
                                ),
                            });

                            // Stretching when having a parent

                            if let Some(st) = stretches.get(*entity) {
                                new_size = Some(match st.stretch {
                                    Stretch::X => {
                                        (parent_global.width - st.margin.0 * 2.0, local.height)
                                    }
                                    Stretch::Y => {
                                        (local.width, parent_global.height - st.margin.1 * 2.0)
                                    }
                                    Stretch::XY => (
                                        parent_global.width - st.margin.0 * 2.0,
                                        parent_global.height - st.margin.1 * 2.0,
                                    ),
                                });
                            }
                        }
                    }
                }
                _ => (),
            }

            // Changing the position and size values here because of how borrowing works.

            if let Some(c) = combined_transform {
                if let Some(local) = locals.get_mut(*entity) {
                    local.global_x = c.0;
                    local.global_y = c.1;
                    local.global_z = c.2;
                }
            }

            if let Some(s) = new_size {
                if let Some(local) = locals.get_mut(*entity) {
                    local.width = s.0;
                    local.height = s.1;
                }
            }
        }

        // When you don't have a parent but do have stretch on, resize with screen size.
        for (entity, mut local, stretch) in (&*entities, &mut locals, &stretches).join() {
            if !parents.contains(entity) {
                let new_size = match stretch.stretch {
                    Stretch::X => (screen_dim.width() - stretch.margin.0 * 2.0, local.height),
                    Stretch::Y => (local.width, screen_dim.height() - stretch.margin.1 * 2.0),
                    Stretch::XY => (
                        screen_dim.width() - stretch.margin.0 * 2.0,
                        screen_dim.height() - stretch.margin.1 * 2.0,
                    ),
                };
                local.width = new_size.0;
                local.height = new_size.1;
            }
        }

        // We need to treat any changes done inside the system as non-modifications, so we read out
        // any events that were generated during the system run
        locals.populate_inserted(
            &mut self.inserted_local_id.as_mut().unwrap(),
            &mut self.local_modified,
        );
        locals.populate_modified(
            &mut self.modified_local_id.as_mut().unwrap(),
            &mut self.local_modified,
        );
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.parent_events_id = Some(res.fetch_mut::<ParentHierarchy>().track());
        let mut locals = WriteStorage::<UiTransform>::fetch(res);
        let mut stretches = WriteStorage::<Stretched>::fetch(res);
        self.inserted_local_id = Some(locals.track_inserted());
        self.modified_local_id = Some(locals.track_modified());
        self.inserted_stretch_id = Some(stretches.track_inserted());
        self.modified_stretch_id = Some(stretches.track_modified());
    }
}
