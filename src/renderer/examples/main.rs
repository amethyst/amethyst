
extern crate cgmath;
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst_renderer;

use gfx::{Device};
use gfx::traits::FactoryExt;

use cgmath::{Point3, Vector3, Matrix4};
use cgmath::{Transform, AffineMatrix3};
use genmesh::generators::SphereUV;
use genmesh::{Quad, Triangulate, MapToVertices, Vertices};

use amethyst_renderer::VertexPosNormal as Vertex;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;


fn build_sphere() -> Vec<Vertex> {
    SphereUV::new(16, 16)
        .vertex(|(x, y, z)| Vertex{pos: [x, y, z], normal: [x, y, z]})
        .triangulate()
        .vertices()
        .collect()
}

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("example renderer".to_string())
        .with_dimensions(800, 600)
        .with_vsync();
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let (width, height) = window.get_inner_size().unwrap();
    let mut combuf = factory.create_command_buffer().into();

    let sphere = build_sphere();
    let (buffer, slice) = factory.create_vertex_buffer(&sphere);

    let view: AffineMatrix3<f32> = Transform::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::deg(45.0f32), 8. / 6., 1.0, 10.0);

    let mut scene = amethyst_renderer::Scene{
        projection: proj.into(),
        view: view.mat.into(),
        entities: vec![]
    };

    for x in -2..3 {
        for y in -2..3 {
            for z in -2..3 {
                let x = x as f32 * 4.;
                let y = y as f32 * 4.;
                let z = z as f32 * 4.;

                scene.entities.push(amethyst_renderer::Entity{
                    buffer: buffer.clone(),
                    slice: slice.clone(),
                    ka: [0.; 4],
                    kd: [1.; 4],
                    transform: Matrix4::from_translation(Vector3::new(x, y, z)).into()
                })
            }
        }
    }

    let mut renderer = amethyst_renderer::Renderer::new(&mut factory);
    'main: loop {
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        renderer.render(&scene, &mut combuf);
        combuf.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}