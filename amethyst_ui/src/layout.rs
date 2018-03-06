use super::UiTransform;
use amethyst_core::Parent;
use amethyst_renderer::ScreenDimensions;
use hibitset::BitSet;
use specs::{Component, Entities, Entity, Fetch, FlaggedStorage, Join, ReadStorage, System,
            VecStorage, WriteStorage};
use std::collections::{HashMap, HashSet};

/// Unused, will be implemented in a future PR.
/// Indicated if the position and margins should be calculated in pixel or
/// relative to their parent size.
#[derive(Debug, Clone)]
pub enum ScaleMode {
    /// Use directly the pixel value.
    Pixel,
    /// Use a proportion (%) of the parent's dimensions (or screen, if there is no parent).
    Percent,
}

/// Indicated where the anchor is, relative to the parent (or to the screen, if there is no parent).
/// Follow a normal english Y,X naming.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum Stretch {
    /// Stretches on the X axis.
    X,
    /// Stretches on the Y axis.
    Y,
    /// Stretches on both axes.
    XY,
}

/// Component indicating that the position of this entity should be relative to the parent's position.
#[derive(Debug, Clone)]
pub struct Anchored {
    /// The `Anchor`
    anchor: Anchor,
    /// Defaults to none.
    /// While the position value in UiTransform will be changed,
    /// this keeps track of the offset from the anchor.
    /// By default, it will automatically be set to the UiTransform position before it gets moved by the layout system.
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
#[derive(Debug, Clone)]
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
    pub fn new(stretch: Stretch) -> Self {
        Stretched {
            stretch,
            margin: (0.0, 0.0),
        }
    }

    /// Adds a margin (spacing) on one or multiple axes.
    pub fn with_margin(mut self, x: f32, y: f32) -> Self {
        self.margin = (x, y);
        self
    }
}

impl Component for Stretched {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

/// Used to initialize the `UiTransform` and `Anchored` offsets when using the `Anchored` component.
pub struct UiLayoutSystem;

impl UiLayoutSystem {
    /// Creates a new UiLayoutSystem.
    pub fn new() -> Self {
        UiLayoutSystem {}
    }
}

impl<'a> System<'a> for UiLayoutSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, Anchored>,
        ReadStorage<'a, Parent>,
        Fetch<'a, ScreenDimensions>,
    );

    fn run(&mut self, (entities, mut transform, mut anchor, parent, screen_dim): Self::SystemData) {
        for (entity, mut tr, mut anchor) in (&*entities, &mut transform, &mut anchor).join() {
            if anchor.offset.is_none() {
                anchor.offset = Some((tr.x, tr.y));

                let norm_offset = anchor.norm_offset();

                // Percent will be implemented in a future PR
                let user_offset = match tr.scale_mode {
                    ScaleMode::Pixel => anchor.offset.unwrap(),
                    ScaleMode::Percent => anchor.offset.unwrap(),
                };

                let middle = (screen_dim.width() / 2.0, screen_dim.height() / 2.0);

                let new_pos_x = middle.0 + norm_offset.0 * screen_dim.width() + user_offset.0;
                let new_pos_y = middle.1 + norm_offset.1 * screen_dim.height() + user_offset.1;
                tr.x = new_pos_x;
                tr.y = new_pos_y;
                if parent.get(entity).is_none() {
                    tr.calculated_x = tr.x;
                    tr.calculated_y = tr.y;
                    tr.calculated_z = tr.z;
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
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<Entity>,

    init: BitSet,
    frame_init: BitSet,

    dead: HashSet<Entity>,
    remove_parent: Vec<Entity>,
}

impl UiParentSystem {
    /// Creates a new UiLayoutSystem.
    pub fn new() -> Self {
        Default::default()
    }

    fn remove(&mut self, index: usize) {
        let entity = self.sorted[index];
        self.sorted.swap_remove(index);
        if let Some(swapped) = self.sorted.get(index) {
            self.indices.insert(*swapped, index);
        }
        self.indices.remove(&entity);
        self.init.remove(index as u32);
    }
}

impl<'a> System<'a> for UiParentSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, Parent>,
        ReadStorage<'a, Anchored>,
        ReadStorage<'a, Stretched>,
        Fetch<'a, ScreenDimensions>,
    );
    fn run(
        &mut self,
        (entities, mut locals, mut parents, anchors, stretch, screen_dim): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("ui_parent_system");

        {
            for (entity, parent) in (&*entities, parents.open().1).join() {
                if parent.entity == entity {
                    self.remove_parent.push(entity);
                }
            }

            for entity in self.remove_parent.iter() {
                eprintln!("Entity was its own parent: {:?}", entity);
                parents.remove(*entity);
            }

            self.remove_parent.clear();
        }

        {
            // Checks for entities with a modified local transform or a modified parent, but isn't initialized yet.
            let filter = locals.open().0 & parents.open().0 & !&self.init; // has a local, parent, and isn't initialized.
            for (entity, _) in (&*entities, &filter).join() {
                self.indices.insert(entity, self.sorted.len());
                self.sorted.push(entity);
                self.frame_init.add(entity.id());
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let entity = self.sorted[index];
            let local_dirty = locals.open().1.flagged(entity);
            let parent_dirty = parents.open().1.flagged(entity);

            let mut combined_transform: Option<(f32, f32, f32)> = None;
            let mut new_size: Option<(f32, f32)> = None;

            match (
                parents.get(entity),
                locals.get(entity),
                self.dead.contains(&entity),
            ) {
                (Some(parent), Some(local), false) => {
                    // Make sure this iteration isn't a child before the parent.
                    if parent_dirty {
                        let mut swap = None;

                        // If the index is none then the parent is an orphan or dead
                        if let Some(parent_index) = self.indices.get(&parent.entity) {
                            if parent_index > &index {
                                swap = Some(*parent_index);
                            }
                        }

                        if let Some(p) = swap {
                            // Swap the parent and child.
                            self.sorted.swap(p, index);
                            self.indices.insert(parent.entity, index);
                            self.indices.insert(entity, p);

                            // Swap took place, re-try this index.
                            continue;
                        }
                    }

                    // Kill the entity if the parent is dead.
                    if self.dead.contains(&parent.entity) || !entities.is_alive(parent.entity) {
                        self.remove(index);
                        let _ = entities.delete(entity);
                        self.dead.insert(entity);

                        // Re-try index because swapped with last element.
                        continue;
                    }

                    // Layouting starts here.

                    if local_dirty || parent_dirty || locals.open().1.flagged(parent.entity)
                        || stretch.open().1.flagged(parent.entity)
                    {

                        // Positioning when having a parent.

                        if let Some(parent_global) = locals.get(parent.entity) {
                            combined_transform = Some(match anchors.get(entity) {
                                Some(anchor) => {
                                    let norm = anchor.norm_offset();
                                    (
                                        parent_global.x + parent_global.width * norm.0
                                            + anchor.offset.unwrap().0,
                                        parent_global.y + parent_global.height * norm.1
                                            + anchor.offset.unwrap().1,
                                        parent_global.z + local.z,
                                    )
                                }
                                None => (
                                    parent_global.x + local.x,
                                    parent_global.y + local.y,
                                    parent_global.z + local.z,
                                ),
                            });

                            // Stretching when having a parent

                            if let Some(st) = stretch.get(entity) {
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
                (_, _, dead @ _) => {
                    // This entity should not be in the sorted list, so remove it.
                    self.remove(index);

                    if !dead && !entities.is_alive(entity) {
                        self.dead.insert(entity);
                    }

                    // Re-try index because swapped with last element.
                    continue;
                }
            }

            // Changing the position and size values here because of how borrowing works.

            if let Some(c) = combined_transform {
                if let Some(local) = locals.get_mut(entity) {
                    local.calculated_x = c.0;
                    local.calculated_y = c.1;
                    local.calculated_z = c.2;
                }
            }

            if let Some(s) = new_size {
                if let Some(local) = locals.get_mut(entity) {
                    local.width = s.0;
                    local.height = s.1;
                }
            }

            index += 1;
        }

        // When you don't have a parent but do have stretch on, resize with screen size.
        for (entity, mut local) in (&*entities, &mut locals).join() {
            if parents.get(entity).is_none() {
                if let Some(st) = stretch.get(entity) {
                    let new_size = match st.stretch {
                        Stretch::X => (screen_dim.width() - st.margin.0 * 2.0, local.height),
                        Stretch::Y => (local.width, screen_dim.height() - st.margin.1 * 2.0),
                        Stretch::XY => (
                            screen_dim.width() - st.margin.0 * 2.0,
                            screen_dim.height() - st.margin.1 * 2.0,
                        ),
                    };
                    local.width = new_size.0;
                    local.height = new_size.1;
                }
            }
        }

        (&mut locals).open().1.clear_flags();
        (&mut parents).open().1.clear_flags();

        for bit in &self.frame_init {
            self.init.add(bit);
        }
        self.frame_init.clear();
        self.dead.clear();
    }
}
