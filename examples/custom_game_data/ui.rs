use amethyst::ecs::prelude::{Entity, World};
use amethyst::ui::{Anchor, Anchored, FontHandle, UiText, UiTransform};
use super::Tag;

pub fn create_load_ui(world: &mut World, font: FontHandle) -> Entity {
    let fps_display = world
        .create_entity()
        .with(UiTransform::new(
            "fps".to_string(),
            100.,
            25.,
            1.,
            200.,
            50.,
            0,
        ))
        .with(UiText::new(
            font.clone(),
            "N/A".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            25.,
        ))
        .with(Anchored::new(Anchor::TopLeft))
        .build();

    world
        .create_entity()
        .with(UiTransform::new(
            "fps".to_string(),
            0.,
            0.,
            1.,
            200.,
            50.,
            0,
        ))
        .with(UiText::new(
            font.clone(),
            "Loading".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            25.,
        ))
        .with(Anchored::new(Anchor::Middle))
        .with(Tag)
        .build();

    fps_display
}

pub fn create_paused_ui(world: &mut World, font: FontHandle) {
    world
        .create_entity()
        .with(UiTransform::new(
            "fps".to_string(),
            0.,
            -50.,
            1.,
            200.,
            50.,
            0,
        ))
        .with(UiText::new(
            font,
            "Paused".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            25.,
        ))
        .with(Anchored::new(Anchor::Middle))
        .with(Tag)
        .build();
}
