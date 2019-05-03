use super::Buffers;
use crate::{error, GltfSceneOptions};
use amethyst_core::math::{Vector3, zero};
use amethyst_error::Error;
use amethyst_rendy::{
    rendy::mesh::{Color, MeshBuilder, Normal, Position, Tangent, TexCoord},
    skinning::JointCombined,
};
use log::trace;
use std::{iter::repeat, ops::Range};

fn compute_if<T, F: Fn() -> T>(predicate: bool, func: F) -> Option<T> {
    if predicate {
        Some(func())
    } else {
        None
    }
}

fn try_compute_if<T, F: Fn() -> Option<T>>(predicate: bool, func: F) -> Option<T> {
    if predicate {
        func()
    } else {
        None
    }
}

enum Indices {
    None,
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Indices {
    fn len(&self) -> Option<usize> {
        match self {
            Indices::None => None,
            Indices::U16(vec) => Some(vec.len()),
            Indices::U32(vec) => Some(vec.len()),
        }
    }

    fn map(&self, face: usize, vert: usize) -> usize {
        match self {
            Indices::None => face * 3 + vert,
            Indices::U16(vec) => vec[face * 3 + vert] as usize,
            Indices::U32(vec) => vec[face * 3 + vert] as usize,
        }
    }
}

pub fn load_mesh(
    mesh: &gltf::Mesh<'_>,
    buffers: &Buffers,
    options: &GltfSceneOptions,
) -> Result<Vec<(MeshBuilder<'static>, Option<usize>, Range<[f32; 3]>)>, Error> {
    trace!("Loading mesh");
    let mut primitives = vec![];

    for primitive in mesh.primitives() {
        trace!("Loading mesh primitive");
        let reader = primitive.reader(|buffer| buffers.buffer(&buffer));
        let mut builder = MeshBuilder::new();

        trace!("Loading indices");
        use gltf::mesh::util::ReadIndices;
        let indices = match reader.read_indices() {
            Some(ReadIndices::U8(iter)) => Indices::U16(iter.map(|i| i as u16).collect()),
            Some(ReadIndices::U16(iter)) => Indices::U16(iter.collect()),
            Some(ReadIndices::U32(iter)) => Indices::U32(iter.collect()),
            None => Indices::None,
        };

        trace!("Loading positions");
        let positions = reader
            .read_positions()
            .ok_or(error::Error::MissingPositions)?
            .map(Position)
            .collect::<Vec<_>>();

        let num_unique_vertices = positions.len();
        let num_faces = indices.len().unwrap_or(num_unique_vertices) / 3;

        let normals = compute_if(options.load_normals || options.load_tangents, || {
            trace!("Loading normals");
            if let Some(normals) = reader.read_normals() {
                normals.map(Normal).collect::<Vec<_>>()
            } else {
                trace!("Calculating normals");
                let mut normals = vec![zero::<Vector3<f32>>(); num_unique_vertices];
                for face in 0..num_faces {
                    let i0 = indices.map(face, 0);
                    let i1 = indices.map(face, 1);
                    let i2 = indices.map(face, 2);
                    let a = Vector3::from(positions[i0].0);
                    let b = Vector3::from(positions[i1].0);
                    let c = Vector3::from(positions[i2].0);
                    let n = (b - a).cross(&(c - a));
                    normals[i0] += n;
                    normals[i1] += n;
                    normals[i2] += n;
                }
                normals.into_iter().map(|n| Normal(n.normalize().into())).collect::<Vec<_>>()
            }
        });

        let tex_coords = compute_if(options.load_texcoords || options.load_tangents, || {
            trace!("Loading texture coordinates");
            if let Some(tex_coords) = reader.read_tex_coords(0).map(|t| t.into_f32()) {
                if options.flip_v_coord {
                    tex_coords
                        .map(|[u, v]| TexCoord([u, 1. - v]))
                        .collect::<Vec<_>>()
                } else {
                    tex_coords.map(TexCoord).collect::<Vec<_>>()
                }
            } else {
                let (u, v) = options.generate_tex_coords;
                let v = if options.flip_v_coord { v } else { 1.0 - v };
                repeat(TexCoord([u, v]))
                    .take(positions.len())
                    .collect::<Vec<_>>()
            }
        });

        let tangents = compute_if(options.load_tangents, || {
            trace!("Loading tangents");
            let tangents = reader.read_tangents();
            match tangents {
                Some(tangents) => tangents.map(Tangent).collect::<Vec<_>>(),
                None => {
                    let normals = normals.as_ref().unwrap();
                    let tex_coords = tex_coords.as_ref().unwrap();
                    let mut tangents = vec![Tangent([0.0, 0.0, 0.0, 0.0]); num_unique_vertices];

                    mikktspace::generate_tangents(
                        &|| 3,
                        &|| num_faces,
                        &|face, vert| &positions[indices.map(face, vert)].0,
                        &|face, vert| &normals[indices.map(face, vert)].0,
                        &|face, vert| &tex_coords[indices.map(face, vert)].0,
                        &mut |face, vert, tangent| {
                            let [x, y, z, w] = tangent;
                            tangents[indices.map(face, vert)] = Tangent([x, y, z, 1.0 - w]);
                        },
                    );
                    tangents
                }
            }
        });

        let colors = try_compute_if(options.load_colors, || {
            trace!("Loading colors");
            if let Some(colors) = reader.read_colors(0) {
                Some(colors.into_rgba_f32().map(Color).collect::<Vec<_>>())
            } else {
                None
            }
        });

        let joints = try_compute_if(options.load_animations, || {
            trace!("Loading animations");
            if let (Some(ids), Some(weights)) = (reader.read_joints(0), reader.read_weights(0)) {
                let zip = ids.into_u16().zip(weights.into_f32());
                let joints = zip
                    .map(|(ids, weights)| JointCombined::new(ids, weights))
                    .collect::<Vec<_>>();

                Some(joints)
            } else {
                None
            }
        });

        match indices {
            Indices::U16(vec) => {
                builder.set_indices(vec);
            }
            Indices::U32(vec) => {
                builder.set_indices(vec);
            }
            Indices::None => {}
        };

        builder.add_vertices(positions);
        normals.map(|v| builder.add_vertices(v));
        tangents.map(|v| builder.add_vertices(v));
        tex_coords.map(|v| builder.add_vertices(v));
        colors.map(|v| builder.add_vertices(v));
        joints.map(|v| builder.add_vertices(v));

        trace!("Loading bounding box");
        let bounds = primitive.bounding_box();
        let bounds = bounds.min..bounds.max;
        let material = primitive.material().index();

        primitives.push((builder, material, bounds));
    }
    trace!("Loaded mesh");
    Ok(primitives)
}
