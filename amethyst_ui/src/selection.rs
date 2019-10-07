use std::marker::PhantomData;

use derivative::Derivative;
use derive_new::new;
use serde::{Deserialize, Serialize};
use winit::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use amethyst_core::{
    ecs::{
        Component, DenseVecStorage, Entities, FlaggedStorage, Join, Read, ReadStorage, ReaderId,
        System, SystemData, World, Write, WriteStorage,
    },
    shrev::EventChannel,
    SystemDesc,
};
use amethyst_derive::SystemDesc;
use amethyst_input::{BindingTypes, InputHandler};

use crate::{CachedSelectionOrder, UiEvent, UiEventType};

// TODO: If none selected and there is a Selectable in the World, select the lower ordered one automatically?

/// Component indicating that a Ui entity is selectable.
/// Generic Type:
/// - G: Selection Group. Used to determine which entities can be selected together at the same time.
#[derive(Debug, Serialize, Deserialize, new)]
pub struct Selectable<G> {
    /// The order in which entities are selected when pressing the `Tab` key or the "go to next" input action.
    pub order: u32,
    #[new(default)]
    /// A multi selection group. When multiple entities are in the same selection group, they can be selected at
    /// the same time by holding shift or control and clicking them.
    /// You can also select the first element, then hold shift and press the keyboard arrow keys.
    // TODO: Holding shift + arrow keys to select more.
    // TODO: Pressing the arrow keys could optionally be binded to change the selected ui element.
    pub multi_select_group: Option<G>,
    #[new(default)]
    /// Indicates if you can select multiple entities at once without having to press the shift or control key.
    pub auto_multi_select: bool,
    /// Indicates if this consumes the inputs. If enabled, all inputs (except Tab) will be ignored when the component is focused.
    /// For example, the arrow keys will not change the selected ui element.
    /// Example usage: Ui Editable Text.
    #[new(default)]
    pub consumes_inputs: bool,
}

impl<G: Send + Sync + 'static> Component for Selectable<G> {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Component indicating that a Ui entity is currently selected.
#[derive(Debug, Serialize, Deserialize)]
pub struct Selected;

impl Component for Selected {
    type Storage = DenseVecStorage<Self>;
}

/// System managing the selection of entities.
/// Reacts to `UiEvent`.
/// Reacts to Tab and Shift+Tab.
#[derive(Debug, SystemDesc)]
#[system_desc(name(SelectionKeyboardSystemDesc))]
pub struct SelectionKeyboardSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    #[system_desc(event_channel_reader)]
    window_reader_id: ReaderId<Event>,
    phantom: PhantomData<G>,
}

impl<G> SelectionKeyboardSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    /// Creates a new `SelectionKeyboardSystem`.
    pub fn new(window_reader_id: ReaderId<Event>) -> Self {
        Self {
            window_reader_id,
            phantom: PhantomData,
        }
    }
}

impl<'a, G> System<'a> for SelectionKeyboardSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        Read<'a, CachedSelectionOrder>,
        WriteStorage<'a, Selected>,
        Write<'a, EventChannel<UiEvent>>,
        Entities<'a>,
    );
    fn run(
        &mut self,
        (window_events, cached, mut selecteds, mut ui_events, entities): Self::SystemData,
    ) {
        /*
        Algorithm in use:

        Add clicked elements + shift + ctrl status.
        If tab or shift-tab
            remove clicked buf
            add replace: select higher or lower id closes to previous highest old id
        if clicked buf isn't empty
            if check currently highest selected multiselect group
                // if shift && ctrl -> shift only
                if shift
                    add multiple
                else if ctrl || auto_multi_select
                    add single
                else
                    add replace
            else
                add replace
        */

        // Checks if tab was pressed.
        // TODO: Controller support/Use InputEvent in addition to keys.
        for event in window_events.read(&mut self.window_reader_id) {
            if let Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Tab),
                                modifiers,
                                ..
                            },
                        ..
                    },
                ..
            } = *event
            {
                // Get index of highest selected ui element
                let highest = cached.highest_order_selected_index(&selecteds);

                if let Some(highest) = highest {
                    // If Some, an element was currently selected. We move the cursor to the next or previous element depending if Shift was pressed.
                    // Select Replace
                    for (entity, _) in (&*entities, &selecteds).join() {
                        ui_events.single_write(UiEvent::new(UiEventType::Blur, entity));
                    }
                    selecteds.clear();

                    let target = if !modifiers.shift {
                        // Up
                        if highest > 0 {
                            cached.cache.get(highest - 1).unwrap_or_else(|| cached.cache.last()
                                .expect("unreachable: A highest ui element was selected, but none exist in the cache."))
                        } else {
                            cached.cache.last()
                                .expect("unreachable: A highest ui element was selected, but none exist in the cache.")
                        }
                    } else {
                        // Down
                        cached.cache.get(highest + 1).unwrap_or_else(|| cached.cache.first()
                        .expect("unreachable: A highest ui element was selected, but none exist in the cache."))
                    };

                    selecteds
                        .insert(target.1, Selected)
                        .expect("unreachable: We are inserting");

                    ui_events.single_write(UiEvent::new(UiEventType::Focus, target.1));
                } else if let Some(lowest) = cached.cache.first() {
                    // If None, nothing was selected. Try to take lowest if it exists.
                    selecteds
                        .insert(lowest.1, Selected)
                        .expect("unreachable: We are inserting");

                    ui_events.single_write(UiEvent::new(UiEventType::Focus, lowest.1));
                }
            }
        }
    }
}

/// Builds a `SelectionMouseSystem`.
#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct SelectionMouseSystemDesc<G, T>
where
    G: Send + Sync + 'static + PartialEq,
    T: BindingTypes,
{
    marker: PhantomData<(G, T)>,
}

impl<'a, 'b, G, T> SystemDesc<'a, 'b, SelectionMouseSystem<G, T>> for SelectionMouseSystemDesc<G, T>
where
    G: Send + Sync + 'static + PartialEq,
    T: BindingTypes,
{
    fn build(self, world: &mut World) -> SelectionMouseSystem<G, T> {
        <SelectionMouseSystem<G, T> as System<'_>>::SystemData::setup(world);

        let ui_reader_id = world.fetch_mut::<EventChannel<UiEvent>>().register_reader();

        SelectionMouseSystem::new(ui_reader_id)
    }
}

/// System handling the clicks on ui entities and selecting them, if applicable.
#[derive(Debug)]
pub struct SelectionMouseSystem<G, T>
where
    T: BindingTypes,
{
    ui_reader_id: ReaderId<UiEvent>,
    phantom: PhantomData<(G, T)>,
}

impl<G, T: BindingTypes> SelectionMouseSystem<G, T>
where
    G: Send + Sync + 'static + PartialEq,
{
    /// Creates a new `SelectionMouseSystem`.
    pub fn new(ui_reader_id: ReaderId<UiEvent>) -> Self {
        Self {
            ui_reader_id,
            phantom: PhantomData,
        }
    }
}

impl<'a, G, T: BindingTypes> System<'a> for SelectionMouseSystem<G, T>
where
    G: Send + Sync + 'static + PartialEq,
{
    type SystemData = (
        Write<'a, EventChannel<UiEvent>>,
        Read<'a, CachedSelectionOrder>,
        WriteStorage<'a, Selected>,
        ReadStorage<'a, Selectable<G>>,
        Read<'a, InputHandler<T>>,
        Entities<'a>,
    );
    fn run(
        &mut self,
        (mut ui_events, cached, mut selecteds, selectables, input_handler, entities): Self::SystemData,
    ) {
        let shift = input_handler.key_is_down(VirtualKeyCode::LShift)
            || input_handler.key_is_down(VirtualKeyCode::RShift);
        let ctrl = input_handler.key_is_down(VirtualKeyCode::LControl)
            || input_handler.key_is_down(VirtualKeyCode::RControl);

        let mut emitted: Vec<UiEvent> = Vec::new();

        // Add clicked elements to clicked buffer
        for ev in ui_events.read(&mut self.ui_reader_id) {
            if let UiEventType::ClickStart = ev.event_type {
                // Ignore events from elements removed between the event emission and now.
                if selectables.get(ev.target).is_some() {
                    let clicked = ev.target;
                    // Inside of the loop because its possible that the user clicks two times in a frame while pressing shift.
                    let highest = cached.highest_order_selected_index(&selecteds);

                    if let Some(highest) = highest {
                        let (highest_is_select, auto_multi_select) = {
                            let highest_multi_select_group = &selectables
                                .get(
                                    cached
                                        .cache
                                        .get(highest)
                                        .expect(
                                            "unreachable: we just got those values from the cache.",
                                        )
                                        .1,
                                )
                                .expect("unreachable: we just got those values from the cache.")
                                .multi_select_group;

                            let (target_multi_select_group, auto_multi_select) = {
                                let target_selectable = selectables.get(clicked).expect("unreachable: Because when filling the buffer we checked that the component still exist on the entity.");
                                (
                                    &target_selectable.multi_select_group,
                                    target_selectable.auto_multi_select,
                                )
                            };
                            (
                                highest_multi_select_group == target_multi_select_group,
                                auto_multi_select,
                            )
                        };

                        if highest_is_select {
                            if shift {
                                // Add from latest selected to target for all that have same multi_select_group
                                let cached_index_clicked = cached.index_of(clicked)
                                    .expect("unreachable: Entity has to be in the cache, otherwise it wouldn't have been added.");

                                // When multi-selecting, you remove everything that was previously selected, and then add everything in the range.
                                for (entity, _) in (&*entities, &selecteds).join() {
                                    emitted.push(UiEvent::new(UiEventType::Blur, entity));
                                }
                                selecteds.clear();

                                let min = cached_index_clicked.min(highest);
                                let max = cached_index_clicked.max(highest);

                                for i in min..=max {
                                    let target_entity = cached.cache.get(i).expect(
                                        "unreachable: Range has to be inside of the cache range.",
                                    );
                                    selecteds
                                        .insert(target_entity.1, Selected)
                                        .expect("unreachable: We are inserting");

                                    emitted.push(UiEvent::new(UiEventType::Focus, target_entity.1));
                                }
                            } else if ctrl || auto_multi_select {
                                // Select adding single element
                                selecteds
                                    .insert(clicked, Selected)
                                    .expect("unreachable: We are inserting");

                                emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                            } else {
                                // Select replace, because we don't want to be adding elements.
                                selecteds.clear();
                                selecteds
                                    .insert(clicked, Selected)
                                    .expect("unreachable: We are inserting");

                                emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                            }
                        } else {
                            for (entity, _) in (&*entities, &selecteds).join() {
                                emitted.push(UiEvent::new(UiEventType::Blur, entity));
                            }
                            // Different multi select group than the latest one selected. Execute Select replace
                            selecteds.clear();

                            selecteds
                                .insert(clicked, Selected)
                                .expect("unreachable: We are inserting");

                            emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                        }
                    } else {
                        // Nothing was previously selected, let's just select single.
                        selecteds
                            .insert(clicked, Selected)
                            .expect("unreachable: We are inserting");

                        emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                    }
                }
            }
        }

        ui_events.iter_write(emitted.into_iter());
    }
}
