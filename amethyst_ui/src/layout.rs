use std::collections::HashSet;

use amethyst_assets::prefab::{serde_diff, SerdeDiff};
use amethyst_core::{
    ecs::*,
    transform::{Children, Parent},
};
use amethyst_window::ScreenDimensions;
use derivative::Derivative;
use glyph_brush::{HorizontalAlign, VerticalAlign};
use serde::{Deserialize, Serialize};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use super::UiTransform;

/// Indicates if the position and margins should be calculated in pixel or
/// relative to their parent size.
#[derive(Derivative, Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, SerdeDiff)]
#[derivative(Default)]
pub enum ScaleMode {
    /// Use directly the pixel value.
    #[derivative(Default)]
    Pixel,
    /// Use a proportion (%) of the parent's dimensions (or screen, if there is no parent).
    Percent,
}

/// Indicated where the anchor is, relative to the parent (or to the screen, if there is no parent).
/// Follow a normal english Y,X naming.
#[derive(Derivative, Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, SerdeDiff)]
#[derivative(Default)]
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
    #[derivative(Default)]
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
    pub fn norm_offset(self) -> (f32, f32) {
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

    /// Vertical align. Used by the `UiGlyphsSystem`.
    pub(crate) fn vertical_align(self) -> VerticalAlign {
        match self {
            Anchor::TopLeft => VerticalAlign::Top,
            Anchor::TopMiddle => VerticalAlign::Top,
            Anchor::TopRight => VerticalAlign::Top,
            Anchor::MiddleLeft => VerticalAlign::Center,
            Anchor::Middle => VerticalAlign::Center,
            Anchor::MiddleRight => VerticalAlign::Center,
            Anchor::BottomLeft => VerticalAlign::Bottom,
            Anchor::BottomMiddle => VerticalAlign::Bottom,
            Anchor::BottomRight => VerticalAlign::Bottom,
        }
    }

    /// Horizontal align. Used by the `UiGlyphsSystem`.
    pub(crate) fn horizontal_align(self) -> HorizontalAlign {
        match self {
            Anchor::TopLeft => HorizontalAlign::Left,
            Anchor::TopMiddle => HorizontalAlign::Center,
            Anchor::TopRight => HorizontalAlign::Right,
            Anchor::MiddleLeft => HorizontalAlign::Left,
            Anchor::Middle => HorizontalAlign::Center,
            Anchor::MiddleRight => HorizontalAlign::Right,
            Anchor::BottomLeft => HorizontalAlign::Left,
            Anchor::BottomMiddle => HorizontalAlign::Center,
            Anchor::BottomRight => HorizontalAlign::Right,
        }
    }
}

/// Indicates if a component should be stretched.
#[derive(Derivative, Debug, Clone, Copy, PartialEq, Deserialize, Serialize, SerdeDiff)]
#[derivative(Default)]
pub enum Stretch {
    /// No stretching occurs
    #[derivative(Default)]
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
        /// Keep the aspect ratio by adding more margin to one axis when necessary
        keep_aspect_ratio: bool,
    },
}

/// Manages the `Parent` component on entities having `UiTransform`
/// It does almost the same as the `TransformSystem`, but with some differences,
/// like `UiTransform` alignment and stretching.
#[derive(Debug)]
pub struct UiTransformSystem {
    screen_size: (f32, f32),
    modified_last_iter: HashSet<Entity>,
}

impl UiTransformSystem {
    /// Creates a new `UiTransformSystem`.
    pub fn new() -> Self {
        Self {
            screen_size: (0.0, 0.0),
            modified_last_iter: HashSet::default(),
        }
    }
}

impl System for UiTransformSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("UiTransformSystem")
                .read_resource::<ScreenDimensions>()
                .with_query(
                    <(Entity, &mut UiTransform)>::query().filter(maybe_changed::<UiTransform>()),
                )
                .with_query(<&mut UiTransform>::query())
                .with_query(<(Entity, &mut Parent)>::query().filter(maybe_changed::<Parent>()))
                .with_query(<(Entity, &Children)>::query())
                .with_query(<(Entity, &mut UiTransform, &Children)>::query())
                .with_query(<(Entity, &mut UiTransform, &Parent)>::query())
                .with_query(
                    <(Entity, &mut UiTransform)>::query()
                        .filter(!component::<Parent>() & !component::<Children>()),
                )
                .build(
                    move |_commands,
                          world,
                          screen_dimensions,
                          (
                        changed_transforms_query,
                        all_transforms_query,
                        children_with_changed_parent,
                        parents_query,
                        transform_with_children_query,
                        transform_with_parent_query,
                        transform_isolated_query,
                    )| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("ui_transform_system");

                        let mut modified_entities: HashSet<Entity> = HashSet::new();

                        changed_transforms_query.for_each_mut(world, |(e, _)| {
                            if !self.modified_last_iter.contains(e) {
                                modified_entities.insert(*e);
                            }
                        });
                        children_with_changed_parent.for_each_mut(world, |(e, _)| {
                            modified_entities.insert(*e);
                        });

                        let current_screen_size =
                            (screen_dimensions.width(), screen_dimensions.height());

                        let screen_resized = current_screen_size != self.screen_size;
                        self.screen_size = current_screen_size;
                        if screen_resized {
                            // Then we process for everyone
                            process_root_iter(
                                transform_with_children_query
                                    .iter_mut(world)
                                    .map(|(_, t, _)| t),
                                &*screen_dimensions,
                            );
                            process_root_iter(
                                transform_isolated_query.iter_mut(world).map(|(_, t)| t),
                                &*screen_dimensions,
                            );
                        } else {
                            // We process only modified
                            process_root_iter(
                                transform_with_children_query
                                    .iter_mut(world)
                                    .filter(|(e, _, _)| modified_entities.contains(e))
                                    .map(|(_, t, _)| t),
                                &*screen_dimensions,
                            );
                            process_root_iter(
                                transform_isolated_query
                                    .iter_mut(world)
                                    .filter(|(e, _)| modified_entities.contains(e))
                                    .map(|(_, t)| t),
                                &*screen_dimensions,
                            );
                        }

                        let (parent_world, mut else_world) = world.split_for_query(parents_query);

                        let modified_children: Vec<(Entity, Entity)> = transform_with_parent_query
                            .iter_mut(&mut else_world)
                            .filter(|(entity, _, parent)| {
                                let self_dirty = modified_entities.contains(&entity);
                                match parents_query.get(&parent_world, parent.0).ok() {
                                    Some((e, _)) => {
                                        let parent_dirty = modified_entities.contains(&e);
                                        parent_dirty || self_dirty || screen_resized
                                    }
                                    None => false,
                                }
                            })
                            .map(|(entity, _, parent)| (*entity, parent.0))
                            .collect();

                        for (entity, parent_entity) in modified_children.iter() {
                            let parent_transform_copy = {
                                if let Ok(transform) =
                                    all_transforms_query.get_mut(world, *parent_entity)
                                {
                                    Some(transform.clone())
                                } else {
                                    None
                                }
                            };

                            let child_transform = all_transforms_query.get_mut(world, *entity).ok();

                            let (mut transform, parent_transform_copy) =
                                match (child_transform, parent_transform_copy) {
                                    (Some(v1), Some(v2)) => (v1, v2),
                                    _ => continue,
                                };
                            let norm = transform.anchor.norm_offset();
                            transform.pixel_x = parent_transform_copy.pixel_x
                                + parent_transform_copy.pixel_width * norm.0;
                            transform.pixel_y = parent_transform_copy.pixel_y
                                + parent_transform_copy.pixel_height * norm.1;
                            transform.global_z = parent_transform_copy.global_z + transform.local_z;

                            let new_size = match transform.stretch {
                                Stretch::NoStretch => (transform.width, transform.height),
                                Stretch::X { x_margin } => {
                                    (
                                        parent_transform_copy.pixel_width - x_margin * 2.0,
                                        transform.height,
                                    )
                                }
                                Stretch::Y { y_margin } => {
                                    (
                                        transform.width,
                                        parent_transform_copy.pixel_height - y_margin * 2.0,
                                    )
                                }
                                Stretch::XY {
                                    keep_aspect_ratio: false,
                                    x_margin,
                                    y_margin,
                                } => {
                                    (
                                        parent_transform_copy.pixel_width - x_margin * 2.0,
                                        parent_transform_copy.pixel_height - y_margin * 2.0,
                                    )
                                }
                                Stretch::XY {
                                    keep_aspect_ratio: true,
                                    x_margin,
                                    y_margin,
                                } => {
                                    let scale = f32::min(
                                        (parent_transform_copy.pixel_width - x_margin * 2.0)
                                            / transform.width,
                                        (parent_transform_copy.pixel_height - y_margin * 2.0)
                                            / transform.height,
                                    );

                                    (transform.width * scale, transform.height * scale)
                                }
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
                            let pivot_norm = transform.pivot.norm_offset();
                            transform.pixel_x += transform.pixel_width * -pivot_norm.0;
                            transform.pixel_y += transform.pixel_height * -pivot_norm.1;
                        }

                        self.modified_last_iter.clear();
                        for e in modified_entities.iter() {
                            self.modified_last_iter.insert(*e);
                        }

                        for (e, _) in modified_children.iter() {
                            self.modified_last_iter.insert(*e);
                        }
                    },
                ),
        )
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
            Stretch::X { x_margin } => (screen_dim.width() - x_margin * 2.0, transform.height),
            Stretch::Y { y_margin } => (transform.width, screen_dim.height() - y_margin * 2.0),
            Stretch::XY {
                keep_aspect_ratio: false,
                x_margin,
                y_margin,
            } => {
                (
                    screen_dim.width() - x_margin * 2.0,
                    screen_dim.height() - y_margin * 2.0,
                )
            }
            Stretch::XY {
                keep_aspect_ratio: true,
                x_margin,
                y_margin,
            } => {
                let scale = f32::min(
                    (screen_dim.width() - x_margin * 2.0) / transform.width,
                    (screen_dim.height() - y_margin * 2.0) / transform.height,
                );

                (transform.width * scale, transform.height * scale)
            }
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
        let pivot_norm = transform.pivot.norm_offset();
        transform.pixel_x += transform.pixel_width * -pivot_norm.0;
        transform.pixel_y += transform.pixel_height * -pivot_norm.1;
    }
}
