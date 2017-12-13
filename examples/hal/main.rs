extern crate amethyst_core as core;
extern crate amethyst_renderer as renderer;
extern crate amethyst_utils as utils;
#[macro_use]
extern crate error_chain;
extern crate genmesh;

extern crate thread_profiler;

extern crate specs;
extern crate winit;

mod flat;

// use amethyst::assets::Loader;
use core::cgmath::{Deg, Matrix4, Point3, Vector3};
use core::cgmath::prelude::InnerSpace;
use core::transform::Transform;
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;
use specs::World;

use renderer::gfx_hal::{Device, Instance, QueueFamily, Surface, Swapchain};
use renderer::gfx_hal::command::{ClearColor, ClearDepthStencil, Rect, Viewport};
use renderer::gfx_hal::device::Extent;
use renderer::gfx_hal::format::{Bgra8, ChannelType, Formatted, Rgba8};
use renderer::gfx_hal::pso::{EntryPoint, Stage};

use renderer::*;
use renderer::cam::{ActiveCamera, Camera};
use renderer::graph::Graph;
use renderer::graph::build::{ColorPin, Merge, PassBuilder, Present};
use renderer::hal::{Hal, HalConfig};
use renderer::memory::Allocator;
use renderer::mesh::{Mesh, MeshBuilder};
use renderer::renderer::{Renderer, RendererConfig};
use renderer::uniform::BasicUniformCache;
use renderer::vertex::PosColor;


use flat::DrawFlat;

error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    #[cfg(feature = "profiler")]
    ::thread_profiler::register_thread_with_profiler("main".into());
    let mut events_loop = winit::EventsLoop::new();

    println!("Init HAL");
    let Hal {
        ref mut device,
        ref mut allocator,
        ref mut center,
        ref mut uploader,
        ref mut renderer,
        ref mut current,
        ref mut shaders,
    } = HalConfig {
        adapter: None,
        arena_size: 1024 * 1024 * 16,
        chunk_size: 1024,
        min_chunk_size: 512,
        compute: false,
        renderer: Some(RendererConfig {
            title: "Amethyst Hal Example",
            width: 1024,
            height: 768,
            events: &events_loop,
        }),
    }.build::<vulkan::Backend>()
        .chain_err(|| "Can't init HAL")?;

    let ref mut renderer = *renderer.as_mut().unwrap();

    shaders.set_shaders_dir(format!("{}/examples/hal", env!("CARGO_MANIFEST_DIR")));

    println!("Build graph");
    let mut graph = {
        let pass = PassBuilder::<vulkan::Backend>::new::<DrawFlat>();

        let passes = [&pass];
        let merge = Merge::new(
            Some(ClearColor::Float([0.15, 0.1, 0.2, 1.0])),
            None,
            &passes,
        );
        let present = Present::new(ColorPin::new(&merge, 0));
        renderer
            .add_graph(ColorPin::new(&merge, 0), &device, allocator, shaders)
            .chain_err(|| "Can't build graph")?
    };

    println!("Create mesh");
    let mesh = {
        let vertices = vec![
            PosColor {
                position: [0.5, -0.5, -0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, -0.5, 0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, 0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, -0.5].into(),
                color: [1.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, -0.5, -0.5].into(),
                color: [1.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, -0.5, 0.5].into(),
                color: [0.0, 1.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, 0.5, 0.5].into(),
                color: [1.0, 0.3, 0.3, 1.0].into(),
            },
            PosColor {
                position: [-0.5, 0.5, -0.5].into(),
                color: [0.3, 1.0, 0.3, 1.0].into(),
            },
        ];
        let indices: Vec<u16> = vec![
            // 0, 1, 3,
            // 1, 2, 3,
            // 1, 0, 5,
            // 0, 4, 5,
            // 0, 3, 4,
            // 3, 7, 4,
            // 4, 7, 5,
            // 7, 6, 5,
            // 2, 6, 3,
            // 6, 7, 3,
            1,
            5,
            2,
            5,
            6,
            2,
        ];
        MeshBuilder::new()
            .with_indices(indices)
            .with_vertices(vertices)
            .build(allocator, uploader, &current, &device)
            .unwrap()
    };

    println!("Fill world");
    let mut world = World::new();
    world.register::<Mesh<vulkan::Backend>>();
    world.register::<Camera>();
    world.register::<Transform>();
    world.register::<BasicUniformCache<vulkan::Backend, flat::TrProjView>>();
    world.register::<flat::Desc<vulkan::Backend>>();

    let cube = world
        .create_entity()
        .with(mesh)
        .with(Transform::default())
        .build();

    let (width, height) = renderer.window.get_inner_size_pixels().unwrap();

    let cam = world
        .create_entity()
        .with(Camera::standard_3d(width as f32, height as f32))
        .with(Transform(Matrix4::look_at(
            [1.0, 1.0, -5.0].into(),
            [0.0, 0.0, 0.0].into(),
            [0.0, 1.0, 0.0].into(),
        )))
        .build();

    world.add_resource(ActiveCamera { entity: cam });

    let mut counter = utils::fps_counter::FPSCounter::new(1024);
    let mut instant = ::std::time::Instant::now();
    let mut last = instant;

    println!("Start rendering");

    let mut first = true;

    for _ in 0..1000 {
        events_loop.poll_events(|_| {});
        renderer.draw(0, current, center, allocator, device, &world);

        let now = ::std::time::Instant::now();
        let delta = now - instant;
        let delta = (delta.as_secs() * 1_000_000_000) + delta.subsec_nanos() as u64;
        counter.push(delta);
        if (now - last) > ::std::time::Duration::from_secs(3) {
            println!("FPS: {}", counter.sampled_fps());
            last = now;
        }
        instant = now;
    }

    world
        .write::<Mesh<vulkan::Backend>>()
        .remove(cube)
        .unwrap()
        .dispose(allocator);

    #[cfg(feature = "profiler")]
    ::thread_profiler::write_profile(&format!("{}/prf.", env!("CARGO_MANIFEST_DIR")));

    ::std::process::exit(0);
    Ok(())
}
