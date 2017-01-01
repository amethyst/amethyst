
extern crate cgmath;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst_renderer;
extern crate rand;

use std::time::SystemTime;
use std::collections::HashMap;
use rand::Rng;

use gfx::{Device};
use gfx::traits::FactoryExt;

use cgmath::{Point3, Vector3, Matrix4, EuclideanVector};
use cgmath::{Transform, AffineMatrix3};
use genmesh::generators::SphereUV;
use genmesh::{Triangulate, MapToVertices, Vertices};

use amethyst_renderer::VertexPosNormal as Vertex;
use amethyst_renderer::framebuffer::{ColorFormat, DepthFormat};
use amethyst_renderer::{Frame, RenderPasses};

fn build_sphere() -> Vec<Vertex> {
    SphereUV::new(32, 32)
        .vertex(|(x, y, z)| Vertex{
            pos: [x, y, z],
            normal: Vector3::new(x, y, z).normalize().into()
        })
        .triangulate()
        .vertices()
        .collect()
}

fn renderpass_gbuffer() -> RenderPasses {
    use amethyst_renderer::pass::*;

    RenderPasses::new("gbuffer",
        vec![
            Clear::new([0., 0., 0., 1.]),
            DrawNoShading::new("main", "main")
        ]
    )
}

fn pipeline_deferred() -> Vec<RenderPasses> {
    use amethyst_renderer::pass::*;

    vec![
        renderpass_gbuffer(),
        RenderPasses::new("main",
            vec![
                BlitLayer::new("gbuffer", "ka"),
                Lighting::new("main", "gbuffer", "main")
            ]
        ),
    ]
}

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Amethyst Renderer Demo".to_string())
        .with_dimensions(800, 600)
        .with_vsync();

    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let combuf = factory.create_command_buffer();

    let sphere = build_sphere();
    let (buffer, slice) = factory.create_vertex_buffer_with_slice(&sphere, ());

    let mut scene = amethyst_renderer::Scene{
        fragments: vec![],
        lights: vec![]
    };

    let mut rng = rand::thread_rng();

    for x in -1..2 {
        for y in -1..2 {
            for z in -1..2 {
                let x = x as f32 * 4.;
                let y = y as f32 * 4.;
                let z = z as f32 * 4.;

                let color = [rng.gen_range(0., 1.), rng.gen_range(0., 1.), rng.gen_range(0., 1.), 1.];

                scene.fragments.push(amethyst_renderer::Fragment{
                    buffer: buffer.clone(),
                    slice: slice.clone(),
                    ka: [color[0] * 0.1, color[1] * 0.1, color[2] * 0.1, 1.],
                    kd: color,
                    transform: Matrix4::from_translation(Vector3::new(x, y, z)).into()
                })
            }
        }
    }

    for x in -2..3 {
        for y in -2..3 {
            for z in -2..3 {
                let x = x as f32 * 5.;
                let y = y as f32 * 5.;
                let z = z as f32 * 5.;

                let r = (x + 10.) / 20.;
                let g = (y + 10.) / 20.;
                let b = (z + 10.) / 20.;

                scene.lights.push(amethyst_renderer::Light{
                    color: [r, g, b, 1.],
                    radius: 1.,
                    center: [x, y, z],
                    propagation_constant: 0.,
                    propagation_linear: 0.,
                    propagation_r_square: 1.,
                })
            }
        }
    }


    let mut frame = Frame{
        passes: vec![],
        framebuffers: HashMap::new(),
        scenes: std::collections::HashMap::new(),
        cameras: std::collections::HashMap::new()
    };

    frame.scenes.insert("main".into(), scene);
    frame.framebuffers.insert(
        "main".into(),
        Box::new(amethyst_renderer::framebuffer::ColorBuffer{
            color: main_color,
            output_depth: main_depth
        }
    ));
    frame.framebuffers.insert(
        "gbuffer".into(),
        Box::new(amethyst_renderer::framebuffer::GeometryBuffer::new(&mut factory, (800, 600)))
    );


    let mut renderer = amethyst_renderer::Renderer::new(combuf);
    renderer.load_all(&mut factory);

    window.set_title("Amethyst Renderer [Deferred]");
    frame.passes = pipeline_deferred();

    let start = SystemTime::now();
    let (mut w, mut h) = (800., 600.);
    'main: loop {
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Key1)) => {
                    window.set_title("Amethyst Renderer [Deferred]");
                    frame.passes = pipeline_deferred();
                }
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Key2)) => {
                    window.set_title("Amethyst Renderer [Deferred [Normal]]");
                    frame.passes = vec![
                        renderpass_gbuffer(),
                        RenderPasses::new("main",
                            vec![amethyst_renderer::pass::BlitLayer::new("gbuffer", "normal")]
                        )
                    ];
                }
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Key3)) => {
                    window.set_title("Amethyst Renderer [Forward Flat]");
                    frame.passes = vec![
                        RenderPasses::new("main",
                            vec![
                                amethyst_renderer::pass::Clear::new([0., 0., 0., 1.]),
                                amethyst_renderer::pass::DrawNoShading::new("main", "main")
                            ]
                        )
                    ];
                }
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Key4)) => {
                    window.set_title("Amethyst Renderer [Forward Wireframe]");
                    frame.passes = vec![
                        RenderPasses::new("main",
                            vec![
                                amethyst_renderer::pass::Clear::new([0., 0., 0., 1.]),
                                amethyst_renderer::pass::Wireframe::new("main", "main")
                            ]
                        )
                    ];
                }
                glutin::Event::Resized(iw, ih) => {
                    {
                        let output = frame.framebuffers.get_mut("main").unwrap();
                        let out = output.downcast_mut::<amethyst_renderer::framebuffer::ColorBuffer<gfx_device_gl::Resources>>();
                        let out = out.unwrap();
                        w = iw as f32;
                        h = ih as f32;
                        gfx_window_glutin::update_views(
                            &window,
                            &mut out.color,
                            &mut out.output_depth
                        );
                    }
                    frame.framebuffers.insert(
                        "gbuffer".into(),
                        Box::new(amethyst_renderer::framebuffer::GeometryBuffer::new(&mut factory, (iw as u16, ih as u16)))
                    );

                }
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        let diff = start.elapsed().unwrap();
        let diff = diff.as_secs() as f32 + diff.subsec_nanos() as f32 / 1e9;
        let view: AffineMatrix3<f32> = Transform::look_at(
            Point3::new(diff.sin() * 6., diff.cos() * 6., 3.0),
            Point3::new(0f32, 0.0, 0.0),
            Vector3::unit_z(),
        );
        let proj = cgmath::perspective(cgmath::deg(60.0f32), w / h, 1.0, 100.0);
        frame.cameras.insert(
            format!("main"),
            amethyst_renderer::Camera{projection: proj.into(), view: view.mat.into()}
        );

        renderer.submit(&frame, &mut device);
        window.swap_buffers().unwrap();
    }
}