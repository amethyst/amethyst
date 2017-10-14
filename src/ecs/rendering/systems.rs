//! Rendering system.
//!

use std::mem;

use assets::AssetStorage;
use shred::Resources;
use shrev::EventChannel;
use specs::common::Errors;
use specs::error::BoxedErr;
use winit::{DeviceEvent, Event, WindowEvent};

use ecs::{Fetch, FetchMut, RunNow, SystemData};
use ecs::rendering::resources::{ScreenDimensions, WindowMessages};
use renderer::{Mesh, Renderer, Texture};
use renderer::formats::{create_mesh_asset, create_texture_asset};
use renderer::pipe::{PipelineData, PolyPipeline};

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem<P> {
    pipe: P,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    cached_size: (u32, u32),
}

impl<P> RenderSystem<P>
where
    P: PolyPipeline,
{
    /// Create a new render system
    pub fn new(pipe: P, renderer: Renderer) -> Self {
        let cached_size = renderer.window().get_inner_size_pixels().unwrap();
        Self {
            pipe,
            renderer,
            cached_size,
        }
    }

    fn do_asset_loading(
        &mut self,
        (errors, mut mesh_storage, mut texture_storage): AssetLoadingData,
    ) {
        mesh_storage.process(
            |mesh_data| {
                Ok(create_mesh_asset(mesh_data, &mut self.renderer)
                    .map_err(|err| BoxedErr::new(err))?)
            },
            &*errors,
        );

        texture_storage.process(
            |texture_data| Ok(create_texture_asset(texture_data, &mut self.renderer)?),
            &*errors,
        );
    }

    fn do_render(
        &mut self,
        (mut event_handler, mut window_messages, mut screen_dimensions, data): RenderData<P>,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_system");
        use std::time::Duration;

        // Process window commands
        for mut command in window_messages.queue.drain() {
            command(self.renderer.window());
        }

        if let Some(size) = self.renderer.window().get_inner_size_pixels() {
            // Send window size changes to the resource
            if size
                != (
                    screen_dimensions.width() as u32,
                    screen_dimensions.height() as u32,
                ) {
                screen_dimensions.update(size.0, size.1);

                // We don't need to send the updated size of the window back to the window itself,
                // so set dirty to false.
                screen_dimensions.dirty = false;
            }
        }

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            self.renderer.window().set_inner_size(
                screen_dimensions.width() as u32,
                screen_dimensions.height() as u32,
            );
            screen_dimensions.dirty = false;
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

type AssetLoadingData<'a> = (
    Fetch<'a, Errors>,
    FetchMut<'a, AssetStorage<Mesh>>,
    FetchMut<'a, AssetStorage<Texture>>,
);

type RenderData<'a, P> = (
    FetchMut<'a, EventChannel<Event>>,
    FetchMut<'a, WindowMessages>,
    FetchMut<'a, ScreenDimensions>,
    <P as PipelineData<'a>>::Data,
);

impl<'a, P> RunNow<'a> for RenderSystem<P>
where
    P: PolyPipeline,
{
    fn run_now(&mut self, res: &'a Resources) {
        self.do_asset_loading(AssetLoadingData::<'a>::fetch(res, 0));
        self.do_render(RenderData::<'a, P>::fetch(res, 0));
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
