//! Displays a shaded sphere to the user.

// extern crate amethyst;
extern crate amethyst_core as core;
extern crate amethyst_renderer_new as renderer;
extern crate amethyst_utils as utils;
extern crate genmesh;
extern crate gfx_backend_vulkan as back;
extern crate gfx_backend_metal as back;
extern crate gfx_hal;
extern crate specs;
extern crate winit;

// use amethyst::assets::Loader;
use core::cgmath::{Deg, Vector3, Matrix4, Point3};
use core::cgmath::prelude::InnerSpace;
use core::transform::Transform;
use specs::World;
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

use gfx_hal::{Device, Instance, QueueFamily, Surface, Swapchain};
use gfx_hal::command::{ClearColor, ClearDepthStencil, Rect, Viewport};
use gfx_hal::device::Extent;
use gfx_hal::format::{Bgra8, ChannelType, Formatted, Rgba8};
use gfx_hal::pso::{EntryPoint, Stage};

use renderer::*;
use renderer::cam::{ActiveCamera, Camera};
use renderer::graph::RenderGraph;
use renderer::graph::build::{PassBuilder, ColorPin, Merge, Present};
use renderer::graph::flat::Flat;
use renderer::memory::DumbAllocator;
use renderer::mesh::{MeshBuilder, Mesh};
use renderer::vertex::PosColor;

fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let wb = winit::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title("hal".to_string());
    let window = wb.build(&events_loop).unwrap();

    let window_size = window.get_inner_size_pixels().unwrap();
    let pixel_width = window_size.0;
    let pixel_height = window_size.1;

    let (_instance, mut adapters, mut surface) = {
        let instance = back::Instance::create("gfx-rs quad", 1);
        let surface = instance.create_surface(&window);
        let adapters = instance.enumerate_adapters();
        (instance, adapters, surface)
    };

    for adapter in &adapters {
        println!("{:?}", adapter.info);
    }
    let adapter = adapters.remove(0);

    let surface_format = surface
        .capabilities_and_formats(&adapter.physical_device)
        .1
        .into_iter()
        .find(|&format| {
            println!("Supported format: {:?}", format);
            format == Bgra8::SELF
        })
        .unwrap();

    let gfx_hal::Gpu {
        device,
        mut queue_groups,
        memory_types,
        ..
    } = adapter.open_with(|family| {
        if family.supports_graphics() && surface.supports_queue_family(family) {
            Some(1)
        } else {
            None
        }
    });

    for memory_type in &memory_types {
        println!("{:?}", memory_type);
    }

    let mut allocator = DumbAllocator::new(memory_types);

    let mut queue_group = gfx_hal::QueueGroup::<_, gfx_hal::Graphics>::new(queue_groups.remove(0));
    let mut command_pool = device.create_command_pool_typed(
        &queue_group,
        gfx_hal::pool::CommandPoolCreateFlags::empty(),
        16,
    ).downgrade();
    let ref mut queue = queue_group.queues[0];

    println!("{:?}", surface_format);
    let swap_config = gfx_hal::window::SwapchainConfig::new().with_color(surface_format);
    let (mut swap_chain, backbuffer) = surface.build_swapchain(swap_config, &queue);

    let vs_module = device
        .create_shader_module_from_glsl(include_str!("hal.vert"), Stage::Vertex)
        .unwrap();

    let fs_module = device
        .create_shader_module_from_glsl(include_str!("hal.frag"), Stage::Fragment)
        .unwrap();

    let mut render = {
        let pass = PassBuilder::<back::Backend>::new::<Flat>(
            EntryPoint {
                entry: "main",
                module: &vs_module,
            },
            EntryPoint {
                entry: "main",
                module: &fs_module,
            },
        );
        let passes = [&pass];
        let merge = Merge::new(Some(ClearColor::Float([0.15, 0.1, 0.2, 1.0])), Some(ClearDepthStencil(0.0, 0)), &passes);
        let present = Present::new(ColorPin::new(&merge, 0));
        RenderGraph::build(present, backbuffer, surface_format, None, Extent { width: pixel_width, height: pixel_height, depth: 1 }, &mut allocator, &device).unwrap()
    };
    println!("{:?}", render);

    let mesh = {
        let vertices = vec![
            PosColor { position: [ 0.5, -0.5, -0.5].into(), color: [1.0, 0.0, 0.0, 1.0].into(), },
            PosColor { position: [ 0.5, -0.5,  0.5].into(), color: [0.0, 1.0, 0.0, 1.0].into(), },
            PosColor { position: [ 0.5,  0.5,  0.5].into(), color: [0.0, 0.0, 1.0, 1.0].into(), },
            PosColor { position: [ 0.5,  0.5, -0.5].into(), color: [1.0, 1.0, 0.0, 1.0].into(), },
            PosColor { position: [-0.5, -0.5, -0.5].into(), color: [1.0, 0.0, 1.0, 1.0].into(), },
            PosColor { position: [-0.5, -0.5,  0.5].into(), color: [0.0, 1.0, 1.0, 1.0].into(), },
            PosColor { position: [-0.5,  0.5,  0.5].into(), color: [1.0, 0.3, 0.3, 1.0].into(), },
            PosColor { position: [-0.5,  0.5, -0.5].into(), color: [0.3, 1.0, 0.3, 1.0].into(), },
        ];
        let indices: Vec<u16> = vec![
            0, 1, 3,
            1, 2, 3,
            1, 0, 5,
            0, 4, 5,
            0, 3, 4,
            3, 7, 4,
            4, 7, 5,
            7, 6, 5,
            2, 6, 3,
            6, 7, 3,
            1, 5, 2,
            5, 6, 2,
        ];
        MeshBuilder::new()
            .with_indices(indices)
            .with_vertices(vertices)
            .build(&mut allocator, &device).unwrap()
    };

    let mut world = World::new();
    world.register::<Mesh<back::Backend>>();
    world.register::<Camera>();
    world.register::<Transform>();

    let cube = world.create_entity()
        .with(mesh)
        .with(Transform::default())
        .build();

    let cam = world.create_entity()
        .with(Camera::standard_3d(pixel_width as f32, pixel_height as f32))
        .with(Transform(Matrix4::look_at([1.0, 1.0, -5.0].into(), [0.0, 0.0, 0.0].into(), [0.0, 1.0, 0.0].into())))
        .build();

    let mut cbufs = vec![];
    let viewport = Viewport {
        rect: Rect {
            x: 0, y: 0, w: pixel_width as u16, h: pixel_height as u16,
        },
        depth: 0.0 .. 1.0,
    };

    let mut counter = utils::fps_counter::FPSCounter::new(1024);
    let mut instant = ::std::time::Instant::now();
    let mut last = instant;
    loop {
        render.draw(&mut command_pool, &mut cbufs, queue, &mut swap_chain, &device, viewport.clone(), &world);
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
}
