

use shrev::EventChannel;
use specs::World;
use winit::{DeviceEvent, Event, EventsLoop, WindowEvent};

pub struct EventsPump(pub EventsLoop);

impl EventsPump {
    pub fn pump_events(&mut self, world: &World) {
        let mut event_handler = world.write_resource::<EventChannel<Event>>();
        let mut events: Vec<Event> = Vec::new();
        self.0.poll_events(|new_event| {
            compress_events(&mut events, new_event);
        });
        event_handler.iter_write(events);
    }
}

/// Input devices can sometimes generate a lot of motion events per frame, these are
/// useless as the extra precision is wasted and these events tend to overflow our
/// otherwise very adequate event buffers.  So this function removes and compresses redundant
/// events.
fn compress_events(vec: &mut Vec<Event>, new_event: Event) {
    use std::mem;

    match new_event {
        Event::WindowEvent { ref event, .. } => match event {
            &WindowEvent::CursorMoved { .. } => {
                let mut iter = vec.iter_mut();
                while let Some(stored_event) = iter.next_back() {
                    match stored_event {
                        &mut Event::WindowEvent {
                            event: WindowEvent::CursorMoved { .. },
                            ..
                        } => {
                            mem::replace(stored_event, new_event.clone());
                            return;
                        }

                        &mut Event::WindowEvent {
                            event: WindowEvent::AxisMotion { .. },
                            ..
                        } => {}

                        &mut Event::DeviceEvent {
                            event: DeviceEvent::Motion { .. },
                            ..
                        } => {}

                        _ => {
                            break;
                        }
                    }
                }
            }

            &WindowEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => {
                let mut iter = vec.iter_mut();
                while let Some(stored_event) = iter.next_back() {
                    match stored_event {
                        &mut Event::WindowEvent {
                            event:
                                WindowEvent::AxisMotion {
                                    axis: stored_axis,
                                    device_id: stored_device,
                                    value: ref mut stored_value,
                                },
                            ..
                        } => if device_id == stored_device && axis == stored_axis {
                            *stored_value += value;
                            return;
                        },

                        &mut Event::WindowEvent {
                            event: WindowEvent::CursorMoved { .. },
                            ..
                        } => {}

                        &mut Event::DeviceEvent {
                            event: DeviceEvent::Motion { .. },
                            ..
                        } => {}

                        _ => {
                            break;
                        }
                    }
                }
            }

            _ => {}
        },

        Event::DeviceEvent {
            device_id,
            event: DeviceEvent::Motion { axis, value },
        } => {
            let mut iter = vec.iter_mut();
            while let Some(stored_event) = iter.next_back() {
                match stored_event {
                    &mut Event::DeviceEvent {
                        device_id: stored_device,
                        event:
                            DeviceEvent::Motion {
                                axis: stored_axis,
                                value: ref mut stored_value,
                            },
                    } => if device_id == stored_device && axis == stored_axis {
                        *stored_value += value;
                        return;
                    },

                    &mut Event::WindowEvent {
                        event: WindowEvent::CursorMoved { .. },
                        ..
                    } => {}

                    &mut Event::WindowEvent {
                        event: WindowEvent::AxisMotion { .. },
                        ..
                    } => {}

                    _ => {
                        break;
                    }
                }
            }
        }

        _ => {}
    }
    vec.push(new_event);
}
