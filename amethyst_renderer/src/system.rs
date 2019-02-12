//! Rendering system.
//!

use amethyst_xr::XRInfo;
use crate::xr::{XRRenderInfo, XRTargetInfo};
use std::{mem, sync::Arc};

use derivative::Derivative;
use log::error;
use rayon::ThreadPool;
use winit::{DeviceEvent, Event, WindowEvent};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::{AssetStorage, HotReloadStrategy};
use amethyst_core::{
    shrev::EventChannel,
    specs::prelude::{Read, ReadExpect, Resources, RunNow, SystemData, Write, WriteExpect},
    Time,
};
use amethyst_error::Error;

use crate::{
    config::DisplayConfig,
    formats::{create_mesh_asset, create_texture_asset},
    mesh::Mesh,
    mtl::{Material, MaterialDefaults},
    pipe::{PipelineBuild, PipelineData, PolyPipeline},
    renderer::Renderer,
    resources::{ScreenDimensions, WindowMessages},
    tex::Texture,
};

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem<P> {
    pipe: P,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    cached_size: (f64, f64),
    // This only exists to allow the system to re-use a vec allocation
    // during event compression.  It's length 0 except during `fn render`.
    event_vec: Vec<Event>,
}

impl<P> RenderSystem<P>
where
    P: PolyPipeline,
{
    /// Build a new `RenderSystem` from the given pipeline builder and config
    pub fn build<B>(pipe: B, config: Option<DisplayConfig>) -> Result<Self, Error>
    where
        B: PipelineBuild<Pipeline = P>,
    {
        use std::env;

        // ask winit explicitly to use X11 since Wayland causes several issues
        // see https://github.com/amethyst/amethyst/issues/890
        env::set_var("WINIT_UNIX_BACKEND", "x11");

        let mut renderer = {
            let mut renderer = Renderer::build();

            if let Some(config) = config.to_owned() {
                renderer.with_config(config);
            }

            renderer.build()?
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
        let cached_size = renderer
            .window()
            .get_inner_size()
            .expect("Window no longer exists")
            .into();
        Self {
            pipe,
            renderer,
            cached_size,
            event_vec: Vec::with_capacity(20),
        }
    }

    fn asset_loading(
        &mut self,
        (time, pool, strategy, mut mesh_storage, mut texture_storage): AssetLoadingData<'_>,
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

    fn window_management(&mut self, (mut window_messages, mut screen_dimensions): WindowData<'_>) {
        // Process window commands
        for mut command in window_messages.queue.drain() {
            command(self.renderer.window());
        }

        let width = screen_dimensions.w;
        let height = screen_dimensions.h;

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            self.renderer
                .window()
                .set_inner_size((width, height).into());
            screen_dimensions.dirty = false;
        }

        let hidpi = self.renderer.window().get_hidpi_factor();

        if let Some(size) = self.renderer.window().get_inner_size() {
            let (window_width, window_height): (f64, f64) = size.to_physical(hidpi).into();

            // Send window size changes to the resource
            if (window_width, window_height) != (width, height) {
                screen_dimensions.update(window_width, window_height);

                // We don't need to send the updated size of the window back to the window itself,
                // so set dirty to false.
                screen_dimensions.dirty = false;
            }
        }
        screen_dimensions.update_hidpi_factor(hidpi);
    }

    fn render(&mut self, res: &'_ Resources) {
        if let Some(mut xr_info) = res.try_fetch_mut::<XRInfo>() {
            {
                let targets = self.renderer.xr_targets().len();
                let target_infos = xr_info.targets();

                if targets != target_infos.len() {
                    self.renderer
                        .init_xr_targets(target_infos.iter().map(|i| i.size.clone()).collect());
                }

                for target_index in 0..targets {
                    let target_info = &target_infos[target_index];
                    *res.fetch_mut::<XRRenderInfo>() = XRRenderInfo::XR(XRTargetInfo {
                        render_target: target_index,
                        view_offset: target_info.view_offset.clone(),
                        projection: target_info.projection.clone(),
                    });
                    self.renderer.draw_to_xr_target(
                        target_index,
                        &mut self.pipe,
                        RenderData::<P>::fetch(res),
                    );
                }
                self.renderer.flush();
            }

            let mut backend = xr_info.backend();
            for (target_index, target) in self.renderer.xr_targets().iter().enumerate() {
                #[cfg(feature = "opengl")]
                {
                    let gl_target = (target.color_bufs()[0].as_input.as_ref().unwrap().raw_view().gl_texture()) as usize;
                    backend.submit_gl_target(target_index, gl_target);
                }
            }

            *res.fetch_mut::<XRRenderInfo>() = XRRenderInfo::Window;
        }
        self.renderer
            .draw_to_window(&mut self.pipe, RenderData::<P>::fetch(res));
        self.renderer.flush();
        self.renderer.swap_window_buffers();

        let events = &mut self.event_vec;
        self.renderer.events_mut().poll_events(|new_event| {
            compress_events(events, new_event);
        });
        res.fetch_mut::<EventChannel<Event>>().iter_write(events.drain(..));
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

type RenderData<'a, P> = <P as PipelineData<'a>>::Data;

impl<'a, P> RunNow<'a> for RenderSystem<P>
where
    P: PolyPipeline,
{
    fn run_now(&mut self, res: &'a Resources) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_system");
        {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_assetloading");
            self.asset_loading(AssetLoadingData::fetch(res));
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_windowmanagement");
            self.window_management(WindowData::fetch(res));
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_render");
            //self.render(RenderData::<P>::fetch(res));
            self.render(res);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        AssetLoadingData::setup(res);
        WindowData::setup(res);
        RenderData::<P>::setup(res);

        let mat = create_default_mat(res);
        res.insert(MaterialDefaults(mat));
        let (width, height) = self
            .renderer
            .window()
            .get_inner_size()
            .expect("Window closed during initialization!")
            .into();
        let hidpi = self.renderer.window().get_hidpi_factor();
        res.insert(ScreenDimensions::new(width, height, hidpi));

        res.insert(XRRenderInfo::Window);
    }
}

fn create_default_mat(res: &mut Resources) -> Material {
    use crate::mtl::TextureOffset;

    use amethyst_assets::Loader;

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
            WindowEvent::CursorMoved { .. } => {
                let mut iter = vec.iter_mut();
                while let Some(stored_event) = iter.next_back() {
                    match stored_event {
                        Event::WindowEvent {
                            event: WindowEvent::CursorMoved { .. },
                            ..
                        } => {
                            mem::replace(stored_event, new_event.clone());
                            return;
                        }

                        Event::WindowEvent {
                            event: WindowEvent::AxisMotion { .. },
                            ..
                        } => {}

                        Event::DeviceEvent {
                            event: DeviceEvent::Motion { .. },
                            ..
                        } => {}

                        _ => {
                            break;
                        }
                    }
                }
            }

            WindowEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => {
                let mut iter = vec.iter_mut();
                while let Some(stored_event) = iter.next_back() {
                    match stored_event {
                        Event::WindowEvent {
                            event:
                                WindowEvent::AxisMotion {
                                    axis: stored_axis,
                                    device_id: stored_device,
                                    value: ref mut stored_value,
                                },
                            ..
                        } => {
                            if device_id == stored_device && axis == stored_axis {
                                *stored_value += value;
                                return;
                            }
                        }

                        Event::WindowEvent {
                            event: WindowEvent::CursorMoved { .. },
                            ..
                        } => {}

                        Event::DeviceEvent {
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
                    Event::DeviceEvent {
                        device_id: stored_device,
                        event:
                            DeviceEvent::Motion {
                                axis: stored_axis,
                                value: ref mut stored_value,
                            },
                    } => {
                        if device_id == *stored_device && axis == *stored_axis {
                            *stored_value += value;
                            return;
                        }
                    }

                    Event::WindowEvent {
                        event: WindowEvent::CursorMoved { .. },
                        ..
                    } => {}

                    Event::WindowEvent {
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
