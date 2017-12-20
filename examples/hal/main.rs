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

use core::cgmath::{Deg, Matrix, Matrix4, Point3, SquareMatrix, Vector3};
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
use renderer::graph::{ColorPin, Graph, Merge, Pass};
use renderer::hal::{Hal, HalConfig, Renderer, RendererConfig};
use renderer::memory::Allocator;
use renderer::mesh::{Mesh, MeshBuilder};
use renderer::uniform::BasicUniformCache;
use renderer::vertex::PosColor;


#[cfg(feature = "gfx-metal")]
use metal::Backend;


#[cfg(feature = "gfx-vulkan")]
use Backend;


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
    }.build::<Backend>()
        .chain_err(|| "Can't init HAL")?;

    let ref mut renderer = *renderer.as_mut().unwrap();

    shaders.set_shaders_dir(format!("{}/examples/hal", env!("CARGO_MANIFEST_DIR")));

    println!("Build graph");
    let mut graph = {
        let pass = [&DrawFlat::build()];
        let merge = Merge::new(
            Some(ClearColor::Float([0.15, 0.1, 0.2, 1.0])),
            Some(ClearDepthStencil(1.0, 0)),
            &pass,
        );
        renderer
            .add_graph(merge.color(0), &device, allocator, shaders)
            .chain_err(|| "Can't build graph")?
    };

    println!("Create mesh");
    let mesh = {
        let vertices = vec![
            // Right
            PosColor {
                position: [0.5, -0.5, -0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, -0.5, 0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, -0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, 0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            // Left
            PosColor {
                position: [-0.5, -0.5, -0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, -0.5, 0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, 0.5, -0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, 0.5, 0.5].into(),
                color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            // Top
            PosColor {
                position: [-0.5, 0.5, -0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, 0.5, 0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, -0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, 0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            // Bottom
            PosColor {
                position: [-0.5, -0.5, -0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, -0.5, 0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, -0.5, -0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, -0.5, 0.5].into(),
                color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            // Front
            PosColor {
                position: [-0.5, -0.5, 0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, 0.5, 0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, -0.5, 0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, 0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            // Back
            PosColor {
                position: [-0.5, -0.5, -0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [-0.5, 0.5, -0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, -0.5, -0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
            PosColor {
                position: [0.5, 0.5, -0.5].into(),
                color: [0.0, 0.0, 1.0, 1.0].into(),
            },
        ];
        let indices: Vec<u16> = vec![
            // Left
            vec![0, 1, 2],
            vec![1, 2, 3],
            // Right
            vec![4, 5, 6],
            vec![5, 6, 7],
            // Top
            vec![8, 9, 10],
            vec![9, 10, 11],
            // Bottom
            vec![12, 13, 14],
            vec![13, 14, 15],
            // Front
            vec![16, 17, 18],
            vec![17, 18, 19],
            // Back
            vec![20, 21, 22],
            vec![21, 22, 23],
        ].into_iter()
            .flat_map(|x| x)
            .collect();
        MeshBuilder::new()
            .with_indices(indices)
            .with_vertices(vertices)
            .build(allocator, uploader, &current, &device)
            .unwrap()
    };

    println!("Fill world");
    let mut world = World::new();
    world.register::<Mesh<Backend>>();
    world.register::<Camera>();
    world.register::<Transform>();
    world.register::<BasicUniformCache<Backend, flat::TrProjView>>();
    world.register::<flat::Desc<Backend>>();

    let cube = world
        .create_entity()
        .with(mesh)
        .with(Transform::default())
        .build();

    let (width, height) = renderer.window.get_inner_size_pixels().unwrap();

    let view = Matrix4::from_translation([0.0, 0.0, -5.0].into());
    let view = view * Matrix4::from_angle_x(Deg(-45.0));
    let view = view * Matrix4::from_angle_y(Deg(25.0));

    let cam = Camera::standard_3d(width as f32, height as f32);

    println!("View: {:?}", view);
    println!("Proj: {:?}", cam.proj);

    let cam = world
        .create_entity()
        .with(cam)
        .with(Transform(view))
        .build();

    world.add_resource(ActiveCamera { entity: cam });

    let mut counter = utils::fps_counter::FPSCounter::new(1024);
    let mut instant = ::std::time::Instant::now();
    let mut last = instant;

    println!("Start rendering");

    let mut first = true;

    for i in 0.. {
        events_loop.poll_events(|_| {});
        renderer.draw(0, current, center, allocator, None, device, &world);

        let now = ::std::time::Instant::now();
        let delta = now - instant;
        let delta = (delta.as_secs() * 1_000_000_000) + delta.subsec_nanos() as u64;
        counter.push(delta);
        if (now - last) > ::std::time::Duration::from_secs(3) {
            println!("FPS: {}, frame: {}", counter.sampled_fps(), i);
            last = now;
        }
        instant = now;
    }

    world
        .write::<Mesh<Backend>>()
        .remove(cube)
        .unwrap()
        .dispose(allocator);

    #[cfg(feature = "profiler")]
    ::thread_profiler::write_profile(&format!("{}/prf.", env!("CARGO_MANIFEST_DIR")));

    ::std::process::exit(0);
    Ok(())
}
