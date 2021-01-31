use std::marker::PhantomData;

use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
};
use amethyst_input::InputHandler;
use derive_new::new;
use serde::{Deserialize, Serialize};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{CachedSelectionOrderResource, UiEvent, UiEventType};

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

/// Component indicating that a Ui entity is currently selected.
#[derive(Debug, Serialize, Deserialize)]
pub struct Selected;

/// System managing the selection of entities.
/// Reacts to `UiEvent`.
/// Reacts to Tab and Shift+Tab.
#[derive(Debug)]
pub struct SelectionKeyboardSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    window_reader_id: ReaderId<Event<'static, ()>>,
    phantom: PhantomData<G>,
}

impl<G> SelectionKeyboardSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    /// Creates a new `SelectionKeyboardSystem`.
    pub fn new(window_reader_id: ReaderId<Event<'static, ()>>) -> Self {
        Self {
            window_reader_id,
            phantom: PhantomData,
        }
    }
}

impl<G> System for SelectionKeyboardSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(SystemBuilder::new("SelectionKeyboardSystem")
            .read_resource::<EventChannel<Event<'static, ()>    >>()
            .read_resource::<CachedSelectionOrderResource>()
            .write_resource::<EventChannel<UiEvent>>()
            .read_resource::<InputHandler>()
            .with_query(<(Entity, &mut Selected)>::query())
            .build(move |commands, world,
                         ( window_events, cached,ui_events, inputs),
                         selected_query| {
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
                                ..
                            },
                            ..
                        },
                        ..
                    } = *event
                    {
                        // Get index of highest selected ui element
                        let highest = cached.highest_order_selected_index(selected_query.iter_mut(world));

                        if let Some(highest) = highest {
                            // If Some, an element was currently selected. We move the cursor to the next or previous element depending if Shift was pressed.
                            // Select Replace
                            selected_query.for_each_mut(world, |(entity, _)| {
                                ui_events.single_write(UiEvent::new(UiEventType::Blur, *entity));
                                commands.remove_component::<Selected>(*entity);
                            });


                            let target = if !inputs.modifiers.shift() {
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
                            commands.add_component(target.1, Selected);

                            ui_events.single_write(UiEvent::new(UiEventType::Focus, target.1));
                        } else if let Some(lowest) = cached.cache.first() {
                            // If None, nothing was selected. Try to take lowest if it exists.
                            commands.add_component(lowest.1, Selected);
                            ui_events.single_write(UiEvent::new(UiEventType::Focus, lowest.1));
                        }
                    }
                }
            })
        )
    }
}

/// System handling the clicks on ui entities and selecting them, if applicable.
#[derive(Debug)]
pub struct SelectionMouseSystem<G> {
    ui_reader_id: ReaderId<UiEvent>,
    phantom: PhantomData<G>,
}

impl<G> SelectionMouseSystem<G>
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

impl<G> System for SelectionMouseSystem<G>
where
    G: Send + Sync + 'static + PartialEq,
{
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("SelectionMouseSystem")
                .read_resource::<CachedSelectionOrderResource>()
                .read_resource::<InputHandler>()
                .write_resource::<EventChannel<UiEvent>>()
                .with_query(<(Entity, &Selectable<G>)>::query())
                .with_query(<(Entity, &mut Selected)>::query())
                .build(move |commands, world, (cached, input_handler, ui_events),
                             (selectables, selected_query)| {
                    let shift = input_handler.key_is_down(VirtualKeyCode::LShift)
                        || input_handler.key_is_down(VirtualKeyCode::RShift);
                    let ctrl = input_handler.key_is_down(VirtualKeyCode::LControl)
                        || input_handler.key_is_down(VirtualKeyCode::RControl);

                    let (selectable_subworld, mut selected_subworld) = world.split_for_query(selectables);

                    let mut emitted: Vec<UiEvent> = Vec::new();
                    // Add clicked elements to clicked buffer
                    for ev in ui_events.read(&mut self.ui_reader_id) {
                        if let UiEventType::ClickStart = ev.event_type {
                            if selectables.get(&selectable_subworld, ev.target).is_err() {
                                selected_query.for_each_mut(&mut selected_subworld, |(entity, _)| {
                                    emitted.push(UiEvent::new(UiEventType::Blur, *entity));
                                    commands.remove_component::<Selected>(*entity);
                                });
                                continue;
                            }

                            let clicked = ev.target;
                            // Inside of the loop because its possible that the user clicks two times in a frame while pressing shift.
                            let highest = cached.highest_order_selected_index(selected_query.iter_mut(&mut selected_subworld));

                            if let Some(highest) = highest {
                                let (highest_is_select, auto_multi_select) = {
                                    let highest_multi_select_group = &selectables
                                        .get(&selectable_subworld,
                                             cached
                                                 .cache
                                                 .get(highest)
                                                 .expect("unreachable: we just got those values from the cache.")
                                                 .1,
                                        )
                                        .expect("unreachable: we just got those values from the cache.")
                                        .1
                                        .multi_select_group;

                                    let (target_multi_select_group, auto_multi_select) = {
                                        let target_selectable = selectables.get(&selectable_subworld, clicked).expect("unreachable: Because when filling the buffer we checked that the component still exist on the entity.");
                                        (
                                            &target_selectable.1.multi_select_group,
                                            target_selectable.1.auto_multi_select,
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
                                        selected_query.for_each_mut(&mut selected_subworld, |(entity, _)| {
                                            emitted.push(UiEvent::new(UiEventType::Blur, *entity));
                                            commands.remove_component::<Selected>(*entity);
                                        });

                                        let min = cached_index_clicked.min(highest);
                                        let max = cached_index_clicked.max(highest);

                                        for i in min..=max {
                                            let target_entity = cached.cache.get(i).expect(
                                                "unreachable: Range has to be inside of the cache range.",
                                            );
                                            commands.add_component(target_entity.1, Selected);
                                            emitted.push(UiEvent::new(UiEventType::Focus, target_entity.1));
                                        }
                                    } else if ctrl || auto_multi_select {
                                        // Select adding single element
                                        commands.add_component(clicked, Selected);
                                        emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                                    } else {
                                        // Select replace, because we don't want to be adding elements.
                                        selected_query.for_each_mut(&mut selected_subworld, |(entity, _)| {
                                            commands.remove_component::<Selected>(*entity);
                                        });
                                        commands.add_component(clicked, Selected);
                                        emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                                    }
                                } else {
                                    selected_query.for_each_mut(&mut selected_subworld, |(entity, _)| {
                                        emitted.push(UiEvent::new(UiEventType::Blur, *entity));
                                        // Different multi select group than the latest one selected. Execute Select replace
                                        commands.remove_component::<Selected>(*entity);
                                    });
                                    commands.add_component(clicked, Selected);
                                    emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                                }
                            } else {
                                commands.add_component(clicked, Selected);
                                emitted.push(UiEvent::new(UiEventType::Focus, clicked));
                            }
                        }
                    }
                })
        )
    }
}
