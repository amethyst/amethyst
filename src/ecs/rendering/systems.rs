//! Rendering system.
//!

use std::mem;

use shrev::EventChannel;
use winit::{DeviceEvent, Event, WindowEvent};

use ecs::{Fetch, FetchMut, System};
use ecs::rendering::resources::{Factory, WindowMessages};
use renderer::Renderer;
use renderer::pipe::{PipelineData, PolyPipeline};

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem<P> {
    pipe: P,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
}

impl<P> RenderSystem<P>
where
    P: PolyPipeline,
{
    /// Create a new render system
    pub fn new(pipe: P, renderer: Renderer) -> Self {
        Self { pipe, renderer }
    }
}

impl<'a, P> System<'a> for RenderSystem<P>
where
    P: PolyPipeline,
{
    type SystemData = (
        Fetch<'a, Factory>,
        FetchMut<'a, EventChannel<Event>>,
        FetchMut<'a, WindowMessages>,
        <P as PipelineData<'a>>::Data,
    );

    fn run(&mut self, (factory, mut event_handler, mut window_messages, data): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_system");
        use std::time::Duration;

        for mut command in window_messages.queue.drain() {
            command(self.renderer.window());
        }

        while let Some(job) = factory.jobs.try_pop() {
            job.exec(&mut self.renderer.factory);
        }

        self.renderer
            .draw(&mut self.pipe, data, Duration::from_secs(0));

        let mut events: Vec<Event> = Vec::new();
        self.renderer.events_mut().poll_events(|new_event| {
            compress_events(&mut events, new_event);
        });

        if let Err(err) = event_handler.slice_write(&events) {
            eprintln!(
                "WARNING: Writing too many window events this frame! {:?}",
                err
            );
        }
    }
}

/// Input devices can sometimes generate a lot of motion events per frame, these are
/// useless as the extra precision is wasted and these events tend to overflow our
/// otherwise very adequate event buffers.  So this function removes and compresses redundant
/// events.
fn compress_events(vec: &mut Vec<Event>, new_event: Event) {
    match new_event {
        Event::WindowEvent { ref event, .. } => match event {
            &WindowEvent::MouseMoved { .. } => {
                let mut iter = vec.iter_mut();
                while let Some(stored_event) = iter.next_back() {
                    match stored_event {
                        &mut Event::WindowEvent {
                            event: WindowEvent::MouseMoved { .. },
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
                            event: WindowEvent::MouseMoved { .. },
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
                        event: WindowEvent::MouseMoved { .. },
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
