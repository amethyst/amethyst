extern crate amethyst;
use amethyst::renderer::{Frontend, Light, Object, Frame};
use amethyst::renderer::types::Buffer;

extern crate amethyst_opengl;
use amethyst_opengl::BackendGl;

fn main() {
    let mut frontend = Frontend::new(BackendGl);

    let mut frame = Frame::new();
    frame.objects = vec![Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite,
                         Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite,
                         Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite,
                         Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite,
                         Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite,
                         Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite,
                         Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite,
                         Object::Mesh { vertices: Buffer::Vertex(0) },
                         Object::Sprite];
    frame.lights = vec![Light::Directional { color: [0.0, 0.0, 0.0, 0.0],
                            direction: [0.0, 0.0, 0.0],
                            intensity: 3.0 } ];

    frontend.draw(frame);
}
