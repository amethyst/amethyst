pub use self::{
    actions::{UiButtonAction, UiButtonActionType},
    builder::{UiButtonBuilder, UiButtonBuilderResources},
    retrigger::{
        UiButtonActionRetrigger, UiButtonActionRetriggerSystem, UiButtonActionRetriggerSystemDesc,
    },
    system::{UiButtonSystem, UiButtonSystemDesc},
};
use crate::{define_widget, Interactable, UiSoundRetrigger, UiText, UiTransform};
use amethyst_assets::Handle;
use amethyst_core::Parent;
use amethyst_rendy::Texture;

mod actions;
mod builder;
mod retrigger;
mod system;

define_widget!(UiButton =>
    entities: [text_entity, image_entity]
    components: [
        (has UiTransform as position on image_entity),
        (has UiTransform as text_position on text_entity),
        (has Handle<Texture> as texture on image_entity),
        (has Interactable as mouse_reactive on image_entity),
        (has UiText as text on text_entity),

        (maybe_has Parent as parent on image_entity),
        (maybe_has UiButtonActionRetrigger as action_retrigger on image_entity),
        (maybe_has UiSoundRetrigger as sound_retrigger on image_entity)
    ]
);
