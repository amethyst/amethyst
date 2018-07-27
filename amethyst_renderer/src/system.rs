//! Rendering system.
//!

use std::mem;
use std::sync::Arc;

use amethyst_assets::{AssetStorage, HotReloadStrategy};
use amethyst_core::shrev::EventChannel;
use amethyst_core::specs::prelude::{
    Read, ReadExpect, Resources, RunNow, SystemData, Write, WriteExpect,
};
use amethyst_core::Time;
use rayon::ThreadPool;
use winit::{DeviceEvent, Event, WindowEvent};

use config::DisplayConfig;
use error::Result;
use formats::{create_mesh_asset, create_texture_asset};
use mesh::Mesh;
use mtl::{Material, MaterialDefaults};
use pipe::{PipelineBuild, PipelineData, PolyPipeline};
use renderer::Renderer;
use resources::{ScreenDimensions, WindowMessages};
use tex::Texture;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem<P> {
    pipe: P,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    cached_size: (u32, u32),
    // This only exists to allow the system to re-use a vec allocation
    // during event compression.  It's length 0 except during `fn render`.
    event_vec: Vec<Event>,
}

impl<P> RenderSystem<P>
where
    P: PolyPipeline,
{
    /// Build a new `RenderSystem` from the given pipeline builder and config
    pub fn build<B>(pipe: B, config: Option<DisplayConfig>) -> Result<Self>
    where
        B: PipelineBuild<Pipeline = P>,
    {
        let mut renderer = {
            let mut renderer = Renderer::build();

            if let Some(config) = config.to_owned() {
                renderer.with_config(config);
            }
            let renderer = renderer.build()?;

            renderer
        };

        match renderer.create_pipe(pipe) {
            Ok(pipe) => Ok(Self::new(pipe, renderer)),
            Err(err) => {
                error!("Failed creating pipeline: {}", err);
                Err(err)
            }
        }
    }

    /// Create a new render system
    pub fn new(pipe: P, renderer: Renderer) -> Self {
        let cached_size = renderer.window().get_inner_size().unwrap();
        Self {
            pipe,
            renderer,
            cached_size,
            event_vec: Vec::with_capacity(20),
        }
    }

    fn asset_loading(
        &mut self,
        (time, pool, strategy, mut mesh_storage, mut texture_storage): AssetLoadingData,
    ) {
        use std::ops::Deref;

        let strategy = strategy.as_ref().map(Deref::deref);

        mesh_storage.process(
            |d| create_mesh_asset(d, &mut self.renderer),
            time.frame_number(),
            &**pool,
            strategy,
        );

        texture_storage.process(
            |d| create_texture_asset(d, &mut self.renderer),
            time.frame_number(),
            &**pool,
            strategy,
        );
    }

    fn window_management(&mut self, (mut window_messages, mut screen_dimensions): WindowData) {
        // Process window commands
        for mut command in window_messages.queue.drain() {
            command(self.renderer.window());
        }

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            self.renderer.window().set_inner_size(
                screen_dimensions.width() as u32,
                screen_dimensions.height() as u32,
            );
            screen_dimensions.dirty = false;
        }

        if let Some(size) = self.renderer.window().get_inner_size() {
            // Send window size changes to the resource
            if size != (
                screen_dimensions.width() as u32,
                screen_dimensions.height() as u32,
            ) {
                screen_dimensions.update(size.0, size.1);

                // We don't need to send the updated size of the window back to the window itself,
                // so set dirty to false.
                screen_dimensions.dirty = false;
            }
        }
    }

    fn render(&mut self, (mut event_handler, data): RenderData<P>) {
        self.renderer.draw(&mut self.pipe, data);
        let events = &mut self.event_vec;
        self.renderer.events_mut().poll_events(|new_event| {
            compress_events(events, new_event);
        });
        event_handler.iter_write(events.drain(..));
    }
}

type AssetLoadingData<'a> = (
    Read<'a, Time>,
    ReadExpect<'a, Arc<ThreadPool>>,
    Option<Read<'a, HotReloadStrategy>>,
    Write<'a, AssetStorage<Mesh>>,
    Write<'a, AssetStorage<Texture>>,
);

type WindowData<'a> = (Write<'a, WindowMessages>, WriteExpect<'a, ScreenDimensions>);

type RenderData<'a, P> = (
    Write<'a, EventChannel<Event>>,
    <P as PipelineData<'a>>::Data,
);

impl<'a, P> RunNow<'a> for RenderSystem<P>
where
    P: PolyPipeline,
{
    fn run_now(&mut self, res: &'a Resources) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_system");
        self.asset_loading(AssetLoadingData::fetch(res));
        self.window_management(WindowData::fetch(res));
        self.render(RenderData::<P>::fetch(res));
    }

    fn setup(&mut self, res: &mut Resources) {
        AssetLoadingData::setup(res);
        WindowData::setup(res);
        RenderData::<P>::setup(res);

        let mat = create_default_mat(res);
        res.insert(MaterialDefaults(mat));
        let (width, height) = self.renderer
            .window()
            .get_inner_size()
            .expect("Window closed during initialization!");
        let hidpi = self.renderer.window().hidpi_factor();
        res.insert(ScreenDimensions::new(width, height, hidpi));
    }
}

fn create_default_mat(res: &mut Resources) -> Material {
    use amethyst_assets::Loader;
    use mtl::TextureOffset;

    let loader = res.fetch::<Loader>();

    let albedo = [0.5, 0.5, 0.5, 1.0].into();
    let emission = [0.0; 4].into();
    let normal = [0.5, 0.5, 1.0, 1.0].into();
    let metallic = [0.0; 4].into();
    let roughness = [0.5; 4].into();
    let ambient_occlusion = [1.0; 4].into();
    let caveat = [1.0; 4].into();

    let tex_storage = res.fetch();

    let albedo = loader.load_from_data(albedo, (), &tex_storage);
    let emission = loader.load_from_data(emission, (), &tex_storage);
    let normal = loader.load_from_data(normal, (), &tex_storage);
    let metallic = loader.load_from_data(metallic, (), &tex_storage);
    let roughness = loader.load_from_data(roughness, (), &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion, (), &tex_storage);
    let caveat = loader.load_from_data(caveat, (), &tex_storage);

    Material {
        alpha_cutoff: 0.01,
        albedo,
        albedo_offset: TextureOffset::default(),
        emission,
        emission_offset: TextureOffset::default(),
        normal,
        normal_offset: TextureOffset::default(),
        metallic,
        metallic_offset: TextureOffset::default(),
        roughness,
        roughness_offset: TextureOffset::default(),
        ambient_occlusion,
        ambient_occlusion_offset: TextureOffset::default(),
        caveat,
        caveat_offset: TextureOffset::default(),
    }
}

/// Input devices can sometimes generate a lot of motion events per frame, these are
/// useless as the extra precision is wasted and these events tend to overflow our
/// otherwise very adequate event buffers.  So this function removes and compresses redundant
/// events.
fn compress_events(vec: &mut Vec<Event>, new_event: Event) {
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
