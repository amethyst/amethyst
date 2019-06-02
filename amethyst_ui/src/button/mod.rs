pub use self::{
    actions::{UiButtonAction, UiButtonActionType},
    builder::{UiButtonBuilder, UiButtonBuilderResources},
    retrigger::{UiButtonActionRetriggerComponent, UiButtonActionRetriggerSystem},
    system::UiButtonSystem,
};
use crate::{
    define_widget, InteractableComponent, UiSoundRetriggerComponent, UiTextComponent,
    UiTransformComponent,
};
use amethyst_assets::Handle;
use amethyst_core::ParentComponent;
use amethyst_rendy::Texture;

mod actions;
mod builder;
mod retrigger;
mod system;

define_widget!(UiButton =>
    entities: [text_entity, image_entity]
    components: [
        (has UiTransformComponent as position on image_entity),
        (has UiTransformComponent as text_position on text_entity),
        (has Handle<Texture> as texture on image_entity),
        (has InteractableComponent as mouse_reactive on image_entity),
        (has UiTextComponent as text on text_entity),

        (maybe_has ParentComponent as parent on image_entity),
        (maybe_has UiButtonActionRetriggerComponent as action_retrigger on image_entity),
        (maybe_has UiSoundRetriggerComponent as sound_retrigger on image_entity)
    ]
);
