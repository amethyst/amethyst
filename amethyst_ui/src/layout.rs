use super::UiTransform;
use amethyst_core::Parent;
use amethyst_core::specs::prelude::{Component, Entities, Entity, Fetch, FlaggedStorage, Join, ReadStorage,
                           System, VecStorage, WriteStorage, ReaderId, InsertedFlag, RemovedFlag, ModifiedFlag, BitSet};
use amethyst_renderer::ScreenDimensions;
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
        Fetch<'a, ScreenDimensions>,
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
                if parent.get(entity).is_none() {
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
pub struct UiParentSystem {
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<Entity>,

    init: BitSet,
    frame_init: BitSet,

    parent_modified: BitSet,
    parent_removed: BitSet,

    local_modified: BitSet,

    stretch_modified: BitSet,

    inserted_parent_id: ReaderId<InsertedFlag>,
    modified_parent_id: ReaderId<ModifiedFlag>,
    removed_parent_id: ReaderId<RemovedFlag>,

    inserted_local_id: ReaderId<InsertedFlag>,
    modified_local_id: ReaderId<ModifiedFlag>,

    inserted_stretch_id: ReaderId<InsertedFlag>,
    modified_stretch_id: ReaderId<ModifiedFlag>,

    dead: HashSet<Entity>,
    remove_parent: Vec<Entity>,
}

impl UiParentSystem {
    /// Creates a new UiLayoutSystem.
    pub fn new(
        inserted_parent_id: ReaderId<InsertedFlag>,
        modified_parent_id: ReaderId<ModifiedFlag>,
        removed_parent_id: ReaderId<RemovedFlag>,
        inserted_local_id: ReaderId<InsertedFlag>,
        modified_local_id: ReaderId<ModifiedFlag>,
        inserted_stretch_id: ReaderId<InsertedFlag>,
        modified_stretch_id: ReaderId<ModifiedFlag>,
    ) -> Self {
        UiParentSystem {
            inserted_parent_id,
            modified_parent_id,
            removed_parent_id,
            inserted_local_id,
            modified_local_id,
            inserted_stretch_id,
            modified_stretch_id,
            indices: HashMap::default(),
            sorted: Vec::default(),
            init: BitSet::default(),
            frame_init: BitSet::default(),
            dead: HashSet::default(),
            remove_parent: Vec::default(),
            parent_modified: BitSet::default(),
            parent_removed: BitSet::default(),
            local_modified: BitSet::default(),
            stretch_modified: BitSet::default(),

        }
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
        (entities, mut locals, mut parents, anchors, stretches, screen_dim): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("ui_parent_system");

        self.parent_modified.clear();
        self.parent_removed.clear();
        self.local_modified.clear();
        self.stretch_modified.clear();

        parents.populate_inserted(&mut self.inserted_parent_id, &mut self.parent_modified);
        parents.populate_modified(&mut self.modified_parent_id, &mut self.parent_modified);
        parents.populate_removed(&mut self.removed_parent_id, &mut self.parent_removed);

        locals.populate_inserted(&mut self.inserted_local_id, &mut self.local_modified);
        locals.populate_modified(&mut self.modified_local_id, &mut self.local_modified);

        stretches.populate_inserted(&mut self.inserted_stretch_id, &mut self.stretch_modified);
        stretches.populate_modified(&mut self.modified_stretch_id, &mut self.stretch_modified);

        {
            for (entity, _, parent) in (&*entities, &self.parent_modified, &parents).join() {
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

        for entity in &self.sorted {
            if self.parent_removed.contains(entity.id()) {
                self.dead.insert(*entity);
            }
        }

        {
            // Checks for entities with a modified local transform or a modified parent, but isn't initialized yet.
            let filter = &self.local_modified & &self.parent_modified & !&self.init; // has a local, parent, and isn't initialized.
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
            let local_dirty = self.local_modified.contains(entity.id());
            let parent_dirty = self.parent_modified.contains(entity.id());

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

                    if local_dirty || parent_dirty || self.stretch_modified.contains(parent.entity.id()) {
                        // Positioning when having a parent.

                        if let Some(parent_global) = locals.get(parent.entity) {
                            combined_transform = Some(match anchors.get(entity) {
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

                            if let Some(st) = stretches.get(entity) {
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
                    local.global_x = c.0;
                    local.global_y = c.1;
                    local.global_z = c.2;
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
        for (entity, mut local, stretch) in (&*entities, &mut locals, &stretches).join() {
            if parents.get(entity).is_none() {
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

        for bit in &self.frame_init {
            self.init.add(bit);
        }
        self.frame_init.clear();
        self.dead.clear();

        // We need to treat any changes done inside the system as non-modifications, so we read out
        // any events that were generated during the system run
        locals.populate_inserted(&mut self.inserted_local_id, &mut self.local_modified);
        locals.populate_modified(&mut self.modified_local_id, &mut self.local_modified);
    }
}
