//! Displays a shaded sphere to the user.

// extern crate amethyst;
extern crate amethyst_renderer_new as renderer;
extern crate genmesh;
extern crate gfx_hal;
extern crate gfx_backend_vulkan as back;
extern crate winit;

// use amethyst::assets::Loader;
// use amethyst::core::cgmath::{Deg, Vector3};
// use amethyst::core::cgmath::prelude::InnerSpace;
// use amethyst::core::transform::Transform;
// use amethyst::ecs::World;
// use amethyst::prelude::*;
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

use gfx_hal::{Device, Instance, QueueFamily, Surface, Swapchain};
use gfx_hal::format::ChannelType;
use gfx_hal::pso::{EntryPoint, Stage};

use renderer::*;
use renderer::graph::flat::Flat;
use renderer::graph::build::PassBuilder;
use renderer::graph::RenderGraph;

fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let wb = winit::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title("hal".to_string());
    let window = wb
        .build(&events_loop)
        .unwrap();

    let window_size = window.get_inner_size_pixels().unwrap();
    let pixel_width = window_size.0 as u16;
    let pixel_height = window_size.1 as u16;

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
        .find(|format| format.1 == ChannelType::Srgb)
        .unwrap();

        let gfx_hal::Gpu { device, mut queue_groups, memory_types, .. } =
        adapter.open_with(|family| {
            if family.supports_graphics() && surface.supports_queue_family(family) {
                Some(1)
            } else { None }
        });

    let mut queue_group = gfx_hal::QueueGroup::<_, gfx_hal::Graphics>::new(queue_groups.remove(0));
    let mut command_pool = device.create_command_pool_typed(&queue_group, gfx_hal::pool::CommandPoolCreateFlags::empty(), 16);
    let mut queue = &mut queue_group.queues[0];

    println!("{:?}", surface_format);
    let swap_config = gfx_hal::window::SwapchainConfig::new()
        .with_color(surface_format);
    let (mut swap_chain, backbuffer) = surface.build_swapchain(swap_config, &queue);

    let vs_module = device
        .create_shader_module_from_glsl(include_str!("hal.vert"), Stage::Vertex)
        .unwrap();

    let fs_module = device
        .create_shader_module_from_glsl(include_str!("hal.frag"), Stage::Fragment)
        .unwrap();

    let pass = PassBuilder::<back::Backend>::new::<Flat>(EntryPoint{ entry: "main", module: &vs_module }, EntryPoint { entry: "main", module: &fs_module });
}
